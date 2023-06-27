use std::str::FromStr;

use anyhow::{anyhow, Context};
use chrono::{Datelike, Days, Local, Months, NaiveDate};

#[derive(Debug, PartialEq, Eq)]
pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    pub fn iter_days(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        self.start.iter_days().take_while(|days| days < &self.end)
    }
}

impl FromStr for DateRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = State::ParsingDate(Default::default());
        let mut s = s;

        let mut dates = FixedLengthArray::<NaiveDate, 2>::default();

        loop {
            if s.is_empty() {
                break;
            }

            let (next_state, remain) = match state {
                State::ParsingOffset(p) => {
                    let (offset, remain) = p.parse_str(s)?;
                    if dates.idx != 1 {
                        return Err(anyhow!("invalid input: {s}"));
                    }
                    let date = match offset {
                        Offset::Days(days) => dates.array[0]
                            .checked_add_days(days)
                            .with_context(|| anyhow!("invalid offset: {days:?}"))?,
                        Offset::Months(months) => dates.array[0]
                            .checked_add_months(months)
                            .with_context(|| anyhow!("invalid offset: {months:?}"))?,
                    };

                    dates.push(date);
                    (State::ParsingOps(Default::default()), remain)
                }
                State::ParsingDate(p) => {
                    let (date, remain) = p.parse_str(s)?;
                    dates
                        .push(date)
                        .with_context(|| anyhow!("find more then two dates"))?;
                    (State::ParsingOps(Default::default()), remain)
                }
                State::ParsingOps(p) => p.parse_str(s)?,
            };

            state = next_state;
            s = remain;
        }

        if dates.idx != 2 {
            Err(anyhow!("failed to find enough dates, input: {s}"))
        } else {
            Ok(DateRange {
                start: dates.array[0],
                end: dates.array[1],
            })
        }
    }
}

enum State {
    ParsingOffset(OffsetParser),
    ParsingDate(DateParser),
    ParsingOps(OpsParser),
}

#[derive(Default)]
struct OpsParser {
    ops: FixedLengthArray<u8, 2>,
}

impl Parser for OpsParser {
    type Output = State;

    fn input(&mut self, c: u8) -> Option<()> {
        match c {
            c @ (b'.' | b'^') => self.ops.push(c),
            _ => None,
        }
    }

    fn finish(self) -> anyhow::Result<Self::Output> {
        match self.ops.array {
            [b'.', b'.'] => Ok(State::ParsingDate(DateParser::default())),
            [b'^', 0] => Ok(State::ParsingOffset(OffsetParser::default())),
            _ => Err(anyhow!("invalid ops: {:?}", self.ops)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Offset {
    Days(Days),
    Months(Months),
}

#[derive(Default)]
struct OffsetParser {
    pending_num: u32,
    unit: u8,
}

impl Parser for OffsetParser {
    type Output = Offset;

    fn input(&mut self, c: u8) -> Option<()> {
        match c {
            n @ (b'0'..=b'9') => (self.unit == 0).then(|| {
                let n = (n - b'0') as u32;
                self.pending_num = self.pending_num * 10 + n;
            }),
            n @ (b'd' | b'm' | b'y') => {
                self.unit = n;
                Some(())
            }
            _ => None,
        }
    }

    fn finish(self) -> anyhow::Result<Self::Output> {
        match self.unit {
            b'd' => Ok(Offset::Days(Days::new(self.pending_num as u64))),
            b'm' => Ok(Offset::Months(Months::new(self.pending_num))),
            b'y' => Ok(Offset::Months(Months::new(self.pending_num * 12))),
            n => Err(anyhow!("invalid offset modifier: {n}")),
        }
    }
}

#[derive(Default)]
struct DateParser {
    nums: FixedLengthArray<u16, 3>,
    pending_num: u16,
}

impl Parser for DateParser {
    type Output = NaiveDate;

    fn input(&mut self, c: u8) -> Option<()> {
        match c {
            b'-' => {
                self.nums.push(self.pending_num)?;
                self.pending_num = 0;
                Some(())
            }
            n @ (b'0'..=b'9') => {
                let n = (n - b'0') as u16;
                self.pending_num = self.pending_num * 10 + n;
                Some(())
            }
            c => None,
        }
    }

    fn finish(mut self) -> anyhow::Result<NaiveDate> {
        self.nums
            .push(self.pending_num)
            .with_context(|| anyhow!("too many nums: {:?}", self.nums))?;
        match self.nums.idx {
            3 => NaiveDate::from_ymd_opt(
                self.nums.array[0] as i32,
                self.nums.array[1] as u32,
                self.nums.array[2] as u32,
            )
            .with_context(|| anyhow!("invalid input: {:?}", self.nums)),
            2 => {
                let now = Local::now();
                NaiveDate::from_ymd_opt(
                    now.naive_local().year(),
                    self.nums.array[0] as u32,
                    self.nums.array[1] as u32,
                )
                .with_context(|| anyhow!("invalid input: {:?}", self.nums))
            }
            1 => {
                let now = Local::now();
                NaiveDate::from_ymd_opt(
                    now.naive_local().year(),
                    now.naive_local().month(),
                    self.nums.array[0] as u32,
                )
                .with_context(|| anyhow!("invalid input: {:?}", self.nums))
            }
            0 => Ok(Local::now().naive_local().date()),
            _ => unreachable!(),
        }
    }
}

trait Parser {
    type Output;

    fn input(&mut self, c: u8) -> Option<()>;

    fn finish(self) -> anyhow::Result<Self::Output>;

    fn parse_str(mut self, s: &str) -> anyhow::Result<(Self::Output, &str)>
    where
        Self: Sized,
    {
        let iter = s.as_bytes().iter().enumerate();
        for (idx, &c) in iter {
            if self.input(c).is_none() {
                return Ok((self.finish()?, &s[idx..]));
            }
        }

        static EMPTY: String = String::new();
        Ok((self.finish()?, &EMPTY))
    }
}

/// const generic 可以消掉index为literal的边界检测
#[derive(Debug)]
struct FixedLengthArray<T: Default, const N: usize> {
    array: [T; N],
    idx: usize,
}

impl<T: Default, const N: usize> Default for FixedLengthArray<T, N> {
    fn default() -> Self {
        Self {
            array: std::array::from_fn(|_| T::default()),
            idx: 0,
        }
    }
}

impl<T: Default, const N: usize> FixedLengthArray<T, N> {
    fn push(&mut self, t: T) -> Option<()> {
        let Some(ptr) = self.array.get_mut(self.idx) else {
            return None;
        };

        *ptr = t;
        self.idx += 1;

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_range() {
        let target_range = DateRange {
            start: NaiveDate::from_str("2020-12-31").unwrap(),
            end: NaiveDate::from_str("2021-12-31").unwrap(),
        };
        let date = "2020-12-31..2021-12-31";
        let range = DateRange::from_str(date).unwrap();
        assert_eq!(range, target_range);

        let date = "2020-12-31^1y";
        let range = DateRange::from_str(date).unwrap();
        assert_eq!(range, target_range);
    }

    #[test]
    fn test_ops_parser() {
        matches!(parse::<OpsParser>("..").unwrap(), State::ParsingDate(_));
        matches!(parse::<OpsParser>("^").unwrap(), State::ParsingOffset(_));

        assert!(parse::<OpsParser>("^.").is_err());
    }

    #[test]
    fn test_offset_parser() {
        assert_eq!(
            parse::<OffsetParser>("1d").unwrap(),
            Offset::Days(Days::new(1)),
        );

        assert_eq!(
            parse::<OffsetParser>("1m").unwrap(),
            Offset::Months(Months::new(1)),
        );

        assert_eq!(
            parse::<OffsetParser>("1y").unwrap(),
            Offset::Months(Months::new(12)),
        );

        assert!(parse::<OffsetParser>("1y1").is_err())
    }

    #[test]
    fn test_complete_date_parser() {
        assert_eq!(
            parse::<DateParser>("2022-01-01").unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("2022-1-1").unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("2022-1-1").unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("2022-1-1").unwrap(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );
    }

    #[test]
    fn test_partial_date_parser() {
        let now = Local::now().naive_local().date();
        assert_eq!(
            parse::<DateParser>("01-01").unwrap(),
            NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("1-1").unwrap(),
            NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("1").unwrap(),
            NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap()
        );

        assert_eq!(
            parse::<DateParser>("01").unwrap(),
            NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap()
        );
    }

    fn parse<P: Parser + Default>(date: &str) -> anyhow::Result<P::Output> {
        let parser = P::default();
        let (date, remian) = parser.parse_str(date)?;

        if remian.is_empty() {
            Ok(date)
        } else {
            eprintln!("remian = {:#?}", remian);
            Err(anyhow!("remain is not empty"))
        }
    }
}

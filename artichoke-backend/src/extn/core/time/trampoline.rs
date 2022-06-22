//! Glue between mruby FFI and `Time` Rust implementation.

use spinoso_time::MICROS_IN_NANO;

use crate::convert::{implicitly_convert_to_int, implicitly_convert_to_string};
use crate::extn::core::symbol::Symbol;
use crate::extn::core::time::{Offset, Time};
use crate::extn::prelude::*;

const MAX_NANOS: i64 = 1_000_000_000 - 1;

// Generate a subsecond multiplier from the given ruby value
//
// - If not provided, the defaults to Micros
// - Otherwise, expects a symbol with :milliseconds, :usec, or :nsec
fn subsec_multiplier(interp: &mut Artichoke, subsec_type: Option<Value>) -> Result<i64, Error> {
    match subsec_type {
        Some(subsec_type) => {
            let subsec_symbol = unsafe { Symbol::unbox_from_value(&mut subsec_type.clone(), interp)? }.bytes(interp);
            if subsec_symbol == b"milliseconds" {
                Ok(1_000_000)
            } else if subsec_symbol == b"usec" {
                Ok(1_000)
            } else if subsec_symbol == b"nsec" {
                Ok(1)
            } else {
                Err(ArgumentError::with_message("unexpected unit. expects :milliseconds, :usec, :nsec").into())
            }
        }
        None => Ok(MICROS_IN_NANO as i64),
    }
}

fn offset_from_options(interp: &mut Artichoke, options: Value) -> Result<Offset, Error> {
    let hash: Vec<(Value, Value)> = interp.try_convert_mut(options)?;
    let tz = hash
        .into_iter()
        .map(|(k, v)| {
            let key = unsafe { Symbol::unbox_from_value(&mut k.clone(), interp)? }.bytes(interp);
            if key == b"in" {
                Ok(v)
            } else {
                Err(ArgumentError::with_message("unknown keyword"))?
            }
        })
        .collect::<Result<Vec<Value>, Error>>()?;

    match tz[..] {
        [mut tz] => match tz.ruby_type() {
            Ruby::Fixnum => {
                let offset_seconds = i32::try_from(implicitly_convert_to_int(interp, tz)?)
                    .map_err(|_| ArgumentError::with_message("invalid offset"))?;
                Ok(Offset::from(offset_seconds))
            }
            Ruby::String => {
                let offset_str = unsafe { implicitly_convert_to_string(interp, &mut tz)? };
                Ok(Offset::try_from(offset_str).map_err(|_| {
                    ArgumentError::with_message("+HH:MM, -HH:MM, UTC, or A..I,K..Z expected for utc_offset")
                })?)
            }
            _ => Err(ArgumentError::with_message(
                "+HH:MM, -HH:MM, UTC, A..I,K..Z, or a signed number of seconds expected for utc_offset",
            ))?,
        },
        _ => Err(ArgumentError::with_message("unknown keyword"))?,
    }
}

// Constructor
pub fn now(interp: &mut Artichoke) -> Result<Value, Error> {
    let now = Time::now().map_err(|_| StandardError::with_message("now is not available"))?;
    let result = Time::alloc_value(now, interp)?;
    Ok(result)
}

pub fn at(
    interp: &mut Artichoke,
    seconds: Value,
    opt1: Option<Value>,
    opt2: Option<Value>,
    opt3: Option<Value>,
) -> Result<Value, Error> {
    let (subsec, subsec_type, options) = match (
        opt1.and_then(|opt| Some(opt.ruby_type())),
        opt2.and_then(|opt| Some(opt.ruby_type())),
        opt3.and_then(|opt| Some(opt.ruby_type())),
    ) {
        (None, None, None) => (None, None, None),
        (Some(Ruby::Hash), None, None) => (None, None, opt1),
        (Some(Ruby::Fixnum), None, None) => (opt1, None, None),
        (Some(_), None, None) => Err(ArgumentError::with_message("expected a number"))?,
        (Some(Ruby::Fixnum), Some(Ruby::Hash), None) => (opt1, None, opt2),
        (Some(Ruby::Fixnum), Some(Ruby::Symbol), None) => (opt1, opt2, None),
        (Some(Ruby::Fixnum), Some(_), None) => Err(ArgumentError::with_message(
            "expected one of [:milliseconds, :usec, :nsec]",
        ))?,
        (Some(Ruby::Fixnum), Some(Ruby::Symbol), Some(Ruby::Hash)) => (opt1, opt2, opt3),
        _ => Err(ArgumentError::with_message("invalid arguments"))?,
    };

    let seconds = implicitly_convert_to_int(interp, seconds)?;

    let subsec_nanos = if let Some(subsec) = subsec {
        let subsec_multiplier = subsec_multiplier(interp, subsec_type)?;
        let subsec = implicitly_convert_to_int(interp, subsec)?
            .checked_mul(subsec_multiplier)
            .ok_or_else(|| ArgumentError::with_message("Time too large"))?;

        // checks to see if the value is inside the valid nanos range of 0..=1_000_000_000
        match subsec {
            0..=MAX_NANOS => Ok(subsec as u32),
            i64::MIN..=-1 => Err(ArgumentError::with_message("subseconds needs to be > 0")),
            _ => Err(ArgumentError::with_message("subseconds outside of range")),
        }?
    } else {
        0
    };

    let offset = match options {
        Some(options) => offset_from_options(interp, options)?,
        _ => Offset::local(),
    };

    if let Ok(time) = Time::with_timespec_and_offset(seconds, subsec_nanos, offset) {
        let result = Time::alloc_value(time, interp)?;
        Ok(result)
    } else {
        Err(ArgumentError::with_message("Time too large").into())
    }
}

pub fn mkutc<T>(interp: &mut Artichoke, args: T) -> Result<Value, Error>
where
    T: IntoIterator<Item = Value>,
{
    let _ = interp;
    let _ignored_while_unimplemented = args.into_iter();
    Err(NotImplementedError::new().into())
}

pub fn mktime<T>(interp: &mut Artichoke, args: T) -> Result<Value, Error>
where
    T: IntoIterator<Item = Value>,
{
    let _ = interp;
    let _ignored_while_unimplemented = args.into_iter();
    Err(NotImplementedError::new().into())
}

// Core

pub fn to_int(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let timestamp = time.to_int();
    Ok(interp.convert(timestamp))
}

pub fn to_float(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let duration = time.to_float();
    Ok(interp.convert_mut(duration))
}

pub fn to_rational(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    // Requires `Rational`
    Err(NotImplementedError::new().into())
}

pub fn cmp(interp: &mut Artichoke, mut time: Value, mut other: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    if let Ok(other) = unsafe { Time::unbox_from_value(&mut other, interp) } {
        let cmp = time.cmp(&other);
        Ok(interp.convert(cmp as i32))
    } else {
        let mut message = String::from("comparison of Time with ");
        message.push_str(interp.inspect_type_name_for_value(other));
        message.push_str(" failed");
        Err(ArgumentError::from(message).into())
    }
}

pub fn eql(interp: &mut Artichoke, mut time: Value, mut other: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    if let Ok(other) = unsafe { Time::unbox_from_value(&mut other, interp) } {
        let cmp = time.eq(&other);
        Ok(interp.convert(cmp))
    } else {
        Ok(interp.convert(false))
    }
}

pub fn hash(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())
}

pub fn initialize<T>(interp: &mut Artichoke, time: Value, args: T) -> Result<Value, Error>
where
    T: IntoIterator<Item = Value>,
{
    let _ = interp;
    let _ = time;
    let _ignored_while_unimplemented = args.into_iter();
    Err(NotImplementedError::new().into())
}

pub fn initialize_copy(interp: &mut Artichoke, time: Value, mut from: Value) -> Result<Value, Error> {
    let from = unsafe { Time::unbox_from_value(&mut from, interp)? };
    let result = *from;
    Time::box_into_value(result, time, interp)
}

// Mutators and converters

pub fn mutate_to_local(interp: &mut Artichoke, time: Value, offset: Option<Value>) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = offset;
    Err(NotImplementedError::new().into())
}

pub fn mutate_to_utc(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let mut obj = unsafe { Time::unbox_from_value(&mut time, interp)? };
    obj.set_utc()
        .map_err(|_| ArgumentError::with_message("could not convert to utc"))?;
    Ok(time)
}

pub fn as_local(interp: &mut Artichoke, time: Value, offset: Option<Value>) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = offset;
    Err(NotImplementedError::new().into())
}

pub fn as_utc(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let utc = time
        .to_utc()
        .map_err(|_| ArgumentError::with_message("could not convert to utc"))?;
    Time::alloc_value(utc, interp)
}

// Inspect

pub fn asctime(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())
}

pub fn to_string(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    interp.try_convert_mut(time.to_string())
}

pub fn to_array(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    // Need to implement `Convert` for timezone offset.
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())
}

// Math

pub fn plus(interp: &mut Artichoke, time: Value, other: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = other;
    Err(NotImplementedError::new().into())
}

pub fn minus(interp: &mut Artichoke, time: Value, other: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = other;
    Err(NotImplementedError::new().into())

    //let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    //let other = if let Ok(other) = unsafe { Time::unbox_from_value(&mut other, interp) } {
    //other
    //} else if let Ok(other) = implicitly_convert_to_int(interp, other) {
    //let _ = other;
    //return Err(NotImplementedError::with_message("Time#- with Integer argument is not implemented").into());
    //} else if let Ok(other) = other.try_convert_into::<f64>(interp) {
    //let _ = other;
    //return Err(NotImplementedError::with_message("Time#- with Float argument is not implemented").into());
    //} else {
    //return Err(TypeError::with_message("can't convert into an exact number").into());
    //};
    //let difference = time.sub(*other);
    //interp.try_convert_mut(difference)
}

// Coarse math

pub fn succ(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())

    //interp.warn(b"warning: Time#succ is obsolete; use time + 1")?;
    //let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    //let next = time + 1;
    //Time::alloc_value(next, interp)
}

pub fn round(interp: &mut Artichoke, time: Value, num_digits: Option<Value>) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = num_digits;
    Err(NotImplementedError::new().into())
}

// Datetime

pub fn second(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let second = time.second();
    let result = interp.convert(second);
    Ok(result)
}

pub fn minute(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let minute = time.minute();
    let result = interp.convert(minute);
    Ok(result)
}

pub fn hour(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let hour = time.hour();
    let result = interp.convert(hour);
    Ok(result)
}

pub fn day(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let day = time.day();
    let result = interp.convert(day);
    Ok(result)
}

pub fn month(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let month = time.month();
    let result = interp.convert(month);
    Ok(result)
}

pub fn year(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let year = time.year();
    let result = interp.convert(year);
    Ok(result)
}

pub fn weekday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let weekday = time.day_of_week();
    let result = interp.convert(weekday);
    Ok(result)
}

pub fn year_day(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let year_day = time.day_of_year();
    let result = interp.convert(year_day);
    Ok(result)
}

pub fn is_dst(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_dst = time.is_dst();
    Ok(interp.convert(is_dst))
}

pub fn timezone(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())
}

pub fn utc_offset(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    Err(NotImplementedError::new().into())
}

// Timezone mode

pub fn is_utc(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_utc = time.is_utc();
    Ok(interp.convert(is_utc))
}

// Day of week

pub fn is_sunday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_sunday = time.day_of_week() == 0;
    let result = interp.convert(is_sunday);
    Ok(result)
}

pub fn is_monday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_monday = time.day_of_week() == 1;
    let result = interp.convert(is_monday);
    Ok(result)
}

pub fn is_tuesday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_tuesday = time.day_of_week() == 2;
    let result = interp.convert(is_tuesday);
    Ok(result)
}

pub fn is_wednesday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_wednesday = time.day_of_week() == 3;
    let result = interp.convert(is_wednesday);
    Ok(result)
}

pub fn is_thursday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_thursday = time.day_of_week() == 4;
    let result = interp.convert(is_thursday);
    Ok(result)
}

pub fn is_friday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_friday = time.day_of_week() == 5;
    let result = interp.convert(is_friday);
    Ok(result)
}

pub fn is_saturday(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let is_saturday = time.day_of_week() == 6;
    let result = interp.convert(is_saturday);
    Ok(result)
}

// Unix time value

pub fn microsecond(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let microsecond = time.microseconds();
    let result = interp.convert(microsecond);
    Ok(result)
}

pub fn nanosecond(interp: &mut Artichoke, mut time: Value) -> Result<Value, Error> {
    let time = unsafe { Time::unbox_from_value(&mut time, interp)? };
    let nanosecond = time.nanoseconds();
    let result = interp.convert(nanosecond);
    Ok(result)
}

pub fn subsec(interp: &mut Artichoke, time: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    // Requires `Rational`
    Err(NotImplementedError::new().into())
}

// Time format

pub fn strftime(interp: &mut Artichoke, time: Value, format: Value) -> Result<Value, Error> {
    let _ = interp;
    let _ = time;
    let _ = format;
    // Requires a parser.
    Err(NotImplementedError::new().into())
}

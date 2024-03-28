use super::{Data, FlightPriority, FlightStatus};
use crate::resources::grpc_geo_types::{GeoLineString, GeoPoint};
use chrono::naive::NaiveDate;
use chrono::{Datelike, Duration, Timelike, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

/// Creates a new [Data] object with fields set with random data
///
/// While the generated dates and numbers are random, all values should
/// still be valid and as realistic as possible.
/// Any UUIDs will be generated, but these fields can always be changed
/// by the caller to provide real existing UUIDs.
pub fn get_data_obj() -> Data {
    _get_data_obj(-90, 90)
}

/// Creates a new [Data] object with random fields and dates set only in the future
///
/// Generated flight_plans will only have dates set in the future
/// Note that this eliminates any flight_plans generated with the `Boarding` or `InFlight` states
///
/// While the generated dates and numbers are random, all values should
/// still be valid and as realistic as possible.
/// Any UUIDs will be generated, but these fields can always be changed
/// by the caller to provide real existing UUIDs.
pub fn get_future_data_obj() -> Data {
    _get_data_obj(1, 90)
}

/// Creates a new [Data] object with random fields and dates set only in the past
///
/// Generated flight_plans will only have dates set in the past
/// Note that this eliminates any flight_plans generated with the `Boarding` or `InFlight` states
///
/// While the generated dates and numbers are random, all values should
/// still be valid and as realistic as possible.
/// Any UUIDs will be generated, but these fields can always be changed
/// by the caller to provide real existing UUIDs.
pub fn get_past_data_obj() -> Data {
    _get_data_obj(-90, -1)
}

fn _get_data_obj(days_from_now_min: i64, days_from_now_max: i64) -> Data {
    let now = Utc::now();
    let now = match NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
        .unwrap_or_else(|| {
            panic!(
                "(_get_data_obj) invalid current date from year [{}], month [{}] and day [{}].",
                now.year(),
                now.month(),
                now.day()
            )
        })
        .and_hms_opt(now.time().hour(), 0, 0)
        .expect("could not set hms to full hour")
        .and_local_timezone(Utc)
        .earliest()
    {
        Some(res) => res,
        None => panic!("(_get_data_obj) Could not get current time for timezone Utc"),
    };
    let mut rng = rand::thread_rng();

    // let's have a minimum of 500 meters and a maximum range of about 200km
    let flight_distance_meters: u32 = rng.gen_range(500..200000);

    let start_point = GeoPoint {
        longitude: 4.9164,
        latitude: 52.37466,
    };

    // Flight straight north
    // Quick and dirty conversion - 111,111 meters ~= 1 degree latitude
    let end_point = GeoPoint {
        longitude: 4.9164,
        latitude: start_point.latitude + flight_distance_meters as f64 / 111111.0,
    };

    let path = GeoLineString {
        points: vec![start_point, end_point],
    };

    // use a somewhat realistic duration based on the flight distance (+/- 100km per hour avg.)
    let avg_speed = rng.gen_range(95..105);
    let flight_duration_hours = flight_distance_meters as i64 / 1000 / avg_speed;

    let departure_date = now
        + Duration::days(rng.gen_range(days_from_now_min..days_from_now_max))
        + Duration::hours(rng.gen_range(0..24))
        + Duration::minutes(
            *[0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55]
                .choose(&mut rng)
                .expect("invalid minutes generated"),
        );
    let arrival_date = departure_date + Duration::hours(flight_duration_hours);

    let flight_plan_submitted =
        Some((departure_date - Duration::days(rng.gen_range(1..90))).into());
    let carrier_ack = flight_plan_submitted.clone();
    let origin_timeslot_start = Some(departure_date.into());
    let origin_timeslot_end = Some((departure_date + Duration::minutes(1)).into());
    let target_timeslot_start = Some((arrival_date - Duration::minutes(1)).into());
    let target_timeslot_end = Some(arrival_date.into());

    let mut flight_status = FlightStatus::Draft as i32;
    let mut flight_release_approval = None;
    let mut approved_by = None;
    let mut actual_departure_time = None;
    if departure_date < now {
        println!(
            "(_get_data_obj) departure_date {} is in the past of now {}",
            departure_date, now
        );
        // we're at least in_flight, so change the status
        flight_status = FlightStatus::InFlight as i32;
        // departure was in the past, set actual departure +/- 6 min
        actual_departure_time =
            Some((departure_date + Duration::seconds(rng.gen_range(-360..360))).into());
        // set release approval 12h to 1h before departure
        flight_release_approval =
            Some((departure_date - Duration::seconds(rng.gen_range(60..43200))).into());
        // if we have an approval date, someone must have approved it
        approved_by = Some(Uuid::new_v4().to_string());
    } else if now.signed_duration_since(departure_date).num_hours() <= 1 {
        println!(
            "(_get_data_obj) now {} departure_date {} are less than 1 hour apart: {:?}",
            now,
            departure_date,
            now.signed_duration_since(departure_date)
        );
        // for now, expect to have a sign off at least 1 hour before scheduled departure
        flight_release_approval = Some((now - Duration::hours(rng.gen_range(2..12))).into());
        // if we have an approval date, someone must have approved it
        approved_by = Some(Uuid::new_v4().to_string());
        // we have an approval, so we're at least ready
        flight_status = FlightStatus::Ready as i32;
    } else if now.signed_duration_since(departure_date).num_seconds() <= 600 {
        println!(
            "(_get_data_obj) now {} departure_date {} are less than 10 mins apart: {:?}",
            now,
            departure_date,
            now.signed_duration_since(departure_date)
        );
        // we're 10 mins from departure, should be boarding
        flight_status = FlightStatus::Boarding as i32;
    } else {
        println!(
            "(_get_data_obj) departure_date {} is in the future of now {}",
            departure_date, now
        );
    }
    let mut actual_arrival_time = None;
    if arrival_date >= now {
        println!(
            "(_get_data_obj) arrival_date {} is in the future of now {}",
            arrival_date, now
        );
    } else {
        println!(
            "(_get_data_obj) arrival_date {} is in the past of now {}",
            arrival_date, now
        );

        // arrival was in the past, set actual arrival +/- 6 min
        actual_arrival_time = Some((arrival_date + Duration::seconds(rng.gen_range(-360..360))).into());
        // we've arrived
        flight_status = FlightStatus::Finished as i32;
    }

    Data {
        session_id: "AETH00001".to_string(),
        pilot_id: Uuid::new_v4().to_string(),
        vehicle_id: Uuid::new_v4().to_string(),
        path: Some(path),
        weather_conditions: Some(String::from("cold and windy")),
        origin_vertiport_id: Some(Uuid::new_v4().to_string()),
        origin_vertipad_id: Uuid::new_v4().to_string(),
        target_vertiport_id: Some(Uuid::new_v4().to_string()),
        target_vertipad_id: Uuid::new_v4().to_string(),
        carrier_ack,
        origin_timeslot_start,
        origin_timeslot_end,
        target_timeslot_start,
        target_timeslot_end,
        actual_departure_time,
        actual_arrival_time,
        flight_release_approval,
        flight_plan_submitted,
        approved_by,
        flight_status,
        flight_priority: FlightPriority::Low as i32,
    }
}

#[test]
fn test_get_past_data_obj() {
    for _ in 1..100 {
        let past_data: Data = get_past_data_obj();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check path is set
        assert!(past_data.path.is_some());

        // Check flight_release_approval is set
        assert!(past_data.flight_release_approval.is_some());

        // Check approved_by is set
        assert!(past_data.approved_by.is_some());

        // Check scheduled_departure is set
        assert!(past_data.origin_timeslot_start.is_some());
        assert!(past_data.origin_timeslot_end.is_some());

        // Check scheduled_departure is in the past
        let scheduled_departure = past_data.origin_timeslot_start.unwrap().seconds;
        assert!(scheduled_departure < now as i64);

        // Check actual_departure_time is set
        assert!(past_data.actual_departure_time.is_some());

        // Check actual_departure_time is in the past
        let actual_departure_time = past_data.actual_departure_time.unwrap().seconds;
        assert!(actual_departure_time < now as i64);

        // Check scheduled_arrival is set
        assert!(past_data.target_timeslot_start.is_some());
        assert!(past_data.target_timeslot_end.is_some());

        // Check scheduled_arrival is in the past
        let scheduled_arrival = past_data.target_timeslot_end.unwrap().seconds;
        assert!(scheduled_arrival < now as i64);

        // Check actual_arrival_time is set
        assert!(past_data.actual_arrival_time.is_some());

        // Check actual_arrival_time is in the past
        let actual_departure_time = past_data.actual_arrival_time.unwrap().seconds;
        assert!(actual_departure_time < now as i64);

        // Check flight_status is FINISHED
        assert!(FlightStatus::try_from(past_data.flight_status) == Ok(FlightStatus::Finished));
    }
}

#[test]
fn test_get_future_data_obj() {
    for _ in 1..100 {
        let future_data: Data = get_future_data_obj();

        // Check scheduled_departure is in the future
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let scheduled_departure = future_data.origin_timeslot_start.unwrap().seconds;
        assert!(scheduled_departure > now as i64);

        // Check actual_departure_time is not set
        assert!(future_data.actual_departure_time.is_none());
    }
}

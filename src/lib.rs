#[macro_use]
extern crate serde_derive;

use std::io::Read;
use geojson::{GeoJson, conversion::TryInto};
use geo_types::{Geometry, Point};
use geo::prelude::*;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub enum PolyType<T>
    where T: num_traits::Num + num_traits::cast::NumCast + Copy + std::cmp::PartialOrd
{
    Polygon(geo_types::Polygon<T>),
    MultiPoly(geo_types::MultiPolygon<T>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TzLookup {
    tzs: Vec<(String, PolyType<f64>)>,
}

impl TzLookup {
    /// build a TzLookup from a geo_json reader
    pub fn new(geo_json: impl Read) -> Result<Self, BuildTzLookupError> {
        let gj: GeoJson = serde_json::from_reader(geo_json)?;
        let tzs: Result<Vec<(String, PolyType<f64>)>, BuildTzLookupError> = if let GeoJson::FeatureCollection(col) = gj {
            col.features.iter()
                .map(|f| {
                    let tzid = if let Some(ref props) = f.properties {
                        match props.get("tzid") {
                            None => Err(BuildTzLookupError::FeaturePropTzidMissing),
                            Some(v) => {
                                match v.as_str() {
                                    None => Err(BuildTzLookupError::FeaturePropTzidNotString),
                                    Some(s) => {
                                        Ok(String::from(s))
                                    },
                                }
                            },
                        }
                    } else {
                        Err(BuildTzLookupError::FeatureHasNoProperties)
                    }?;
                    let geo: Geometry<f64> = if let Some(ref geometry) = f.geometry {
                        match geometry.value.clone().try_into() {
                            Ok(g) => Ok(g),
                            Err(e) => Err(BuildTzLookupError::InvalidGeometry(e)),
                        }
                    } else {
                        return Err(BuildTzLookupError::InvalidGeoJson);
                    }?;
                    let geo = match geo {
                        Geometry::MultiPolygon(p) => Ok(PolyType::MultiPoly(p)),
                        Geometry::Polygon(p) => Ok(PolyType::Polygon(p)),
                        _ => Err(BuildTzLookupError::NonPolygonType),
                    }?;
                    Ok((tzid, geo))
                }).collect()
        } else {
            return Err(BuildTzLookupError::InvalidGeoJson);
        };
        Ok(TzLookup{
            tzs: tzs?,
        })
    }

    /// look up a location
    pub fn lookup(&self, lon: f64, lat: f64) -> Option<&str> {
        for tz in self.tzs.iter() {
            match tz.1 {
                PolyType::Polygon(ref p) => {
                    if p.contains(&Point::new(lat, lon)) {
                        return Some(tz.0.as_str());
                    }
                },
                PolyType::MultiPoly(ref mp) => {
                    for p in mp.0.iter() {
                        if p.contains(&Point::new(lat, lon)) {
                            return Some(tz.0.as_str());
                        }
                    }
                }
            }
        }
        return None;
    }

    #[cfg(feature = "inline_tzdata_complete")]
    /// load TzLookup from inline tzdata
    pub fn from_inline_complete() -> Self {
        use bincode::deserialize_from;
        use xz2::read::XzDecoder;

        const X: &'static [u8] = include_bytes!("tzdata_complete.bin.xz");
        let i = XzDecoder::new(X);
        deserialize_from(i).unwrap()
    }
}

#[derive(Debug)]
pub enum BuildTzLookupError {
    ParseError(serde_json::error::Error),
    InvalidGeoJson,
    FeatureHasNoProperties,
    FeaturePropTzidMissing,
    FeaturePropTzidNotString,
    InvalidGeometry(geojson::Error),
    NonPolygonType,
}

impl Error for BuildTzLookupError {}

impl std::fmt::Display for BuildTzLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<serde_json::error::Error> for BuildTzLookupError {
    fn from(e: serde_json::error::Error) -> Self {
        BuildTzLookupError::ParseError(e)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    #[cfg(feature = "inline_tzdata_complete")]
    mod inline_tests {
        use crate::TzLookup;

        #[test]
        fn inline_works() {
            TzLookup::from_inline_complete();
        }

        #[test]
        fn get_coords_test() {
            let tz_lookup = TzLookup::from_inline_complete();
            assert_eq!(Some("America/Los_Angeles"), tz_lookup.lookup(34.075822, -118.522885));
        }
    }
}

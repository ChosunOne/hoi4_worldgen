use crate::components::day_month::DayMonth;
use crate::components::wrappers::{
    ProvinceId, SnowLevel, StrategicRegionId, StrategicRegionName, Temperature, Weight,
};
use crate::{LoadObject, MapError};
use jomini::{JominiDeserialize, TextDeserializer};
use log::{debug, error, info, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Defines a raw strategic region
#[derive(Debug, Clone, JominiDeserialize, Serialize)]
#[non_exhaustive]
pub struct RawStrategicRegion {
    /// The parsed strategic region
    pub strategic_region: StrategicRegion,
}

/// Defines a strategic region
#[derive(Debug, Clone, JominiDeserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct StrategicRegion {
    /// The id of the region
    pub id: StrategicRegionId,
    /// The logical name of the region
    pub name: StrategicRegionName,
    /// The provinces in the region
    pub provinces: Vec<ProvinceId>,
    /// The weather for the region
    pub weather: Weather,
}

/// Container for the weather periods
#[derive(Debug, Clone, JominiDeserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Weather {
    /// The weather periods
    #[jomini(duplicated)]
    pub period: Vec<Period>,
}

/// Defines the weather during a period of time
/// Each strategic region has a weather scope that determines how the weather changes for provinces within it.
/// Each weather system is defined within a period scope within the weather scope.
/// * between scope determines when the weather system occurs, the notation is day.month day.month,
/// i.e. 0.0 30.0 means the weather system occurs between the 1st of January and the 31st, including
/// these days. Note that the first day and the first month are marked as 0, not as 1.
/// * temperature scope determines the minimum and maximum temperature for the weather system.
/// * temperature_day_night scope determines the minimum and maximum temperature variability during
/// day and night for the weather system.
/// * min_snow_level scope determines the minimum amount of snow that is always present in the
/// weather system. Typically only used for areas with year-round snow.  
/// Each of the weather states are given a weight, determining how likely the state will occur
/// within the weather system. The weather states can be found in `/Hearts of Iron IV/common/weather.txt`.
/// TODO: Don't hardcode weather states, instead load them from `weather.txt`
#[derive(Debug, Clone, JominiDeserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Period {
    /// The start and end dates of the period
    pub between: Vec<DayMonth>,
    /// The temperature during the period
    pub temperature: Vec<Temperature>,
    /// The chance that nothing happens during the period
    pub no_phenomenon: Weight,
    /// The chance for light rain
    pub rain_light: Weight,
    /// The chance for heavy rain
    pub rain_heavy: Weight,
    /// The chance for snow
    pub snow: Weight,
    /// The chance for a blizzard
    pub blizzard: Weight,
    /// The chance for arctic water
    pub arctic_water: Weight,
    /// The chance for mud
    pub mud: Weight,
    /// The chance for a sandstorm
    pub sandstorm: Weight,
    /// The minimum snow level during the period
    pub min_snow_level: SnowLevel,
}

/// A map of the strategic regions by id
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct StrategicRegions {
    /// The strategic regions
    pub strategic_regions: HashMap<StrategicRegionId, StrategicRegion>,
}

impl StrategicRegions {
    /// Checks if a file looks like a strategic region file.  Strategic region files should have the
    /// form: `X-StrategicRegion.txt` where X is the strategic region id.
    fn verify_strategic_region_file_name(path: &Path) -> Result<(), MapError> {
        if let Some(filename) = path.file_name() {
            let (id, name) = Self::get_strategic_region_id_and_filename(filename)?;
            if id < StrategicRegionId(1) || name != "StrategicRegion.txt" {
                warn!(
                    "Strategic region file name is not correct: {}",
                    filename.to_string_lossy()
                );
            }
        } else {
            warn!(
                "Strategic region file name is not correct: {}",
                path.to_string_lossy()
            );
        }

        Ok(())
    }

    /// Gets the strategic region id and filename from a file name.
    fn get_strategic_region_id_and_filename(
        filename: &OsStr,
    ) -> Result<(StrategicRegionId, String), MapError> {
        let name_parts = filename
            .to_str()
            .ok_or_else(|| {
                MapError::InvalidStrategicRegionFileName(filename.to_string_lossy().to_string())
            })?
            .split('-')
            .collect::<Vec<_>>();
        let id = name_parts
            .get(0)
            .ok_or_else(|| {
                MapError::InvalidStrategicRegionFileName(filename.to_string_lossy().to_string())
            })?
            .parse::<StrategicRegionId>()?;
        let name = (*name_parts.get(1).ok_or_else(|| {
            MapError::InvalidStrategicRegionFileName(filename.to_string_lossy().to_string())
        })?)
        .to_owned();
        Ok((id, name))
    }

    /// Creates a new map of strategic regions from the `strategicregions` directory.  
    /// # Errors
    /// If the directory cannot be read.
    #[inline]
    pub fn from_dir(path: &Path) -> Result<Self, MapError> {
        let strategic_region_files = fs::read_dir(path)?;
        let mut strategic_regions = HashMap::new();
        for strategic_region_file_result in strategic_region_files {
            let strategic_region_file = strategic_region_file_result?;
            let strategic_region_path = strategic_region_file.path();
            // Check if the file looks like a strategic region
            Self::verify_strategic_region_file_name(&strategic_region_path)?;
            let (filename_id, _) =
                Self::get_strategic_region_id_and_filename(&strategic_region_file.file_name())?;

            let raw_strategic_region = RawStrategicRegion::load_object(&strategic_region_path)?;
            let strategic_region = raw_strategic_region.strategic_region;
            let id = strategic_region.id;

            if id == StrategicRegionId(0) {
                return Err(MapError::InvalidStrategicRegion(id));
            }
            if strategic_region.name == StrategicRegionName("".to_owned()) {
                return Err(MapError::InvalidStrategicRegionName(strategic_region.name));
            }

            if id != filename_id {
                return Err(MapError::InvalidStrategicRegionFileName(
                    strategic_region_path.to_string_lossy().to_string(),
                ));
            }

            strategic_regions.insert(id, strategic_region);
        }

        Ok(Self { strategic_regions })
    }
}

#[allow(clippy::expect_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::panic)]
#[allow(clippy::default_numeric_fallback)]
#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;
    use jomini::TextDeserializer;
    use std::fs;
    use std::path::Path;
    use std::str::FromStr;

    #[test]
    fn it_reads_a_strategic_region_from_a_file() {
        let path = Path::new("./test/map/strategicregions/1-StrategicRegion.txt");
        let raw_strategic_region = RawStrategicRegion::load_object(path).unwrap();
        let strategic_region = raw_strategic_region.strategic_region;
        println!("{:?}", strategic_region);
        assert_eq!(
            strategic_region,
            StrategicRegion {
                id: StrategicRegionId(1),
                name: StrategicRegionName("REGION_1".to_owned()),
                provinces: vec![
                    2, 6, 8, 9, 13, 15, 17, 18, 20, 21, 23, 24, 35, 92, 130, 142, 148, 159, 161,
                    227, 232, 234, 237, 240, 241, 242, 243, 244, 245, 249, 250, 256, 302, 307, 317,
                    426, 429, 430, 431, 435, 437, 440, 448, 455, 524, 549, 612, 614, 615, 619, 622,
                    625, 626, 629, 631, 636, 650, 651, 653, 655, 657, 658, 661, 663, 664, 665, 667,
                    669, 670, 674, 675, 677, 678, 686, 688, 689, 690, 699, 700, 701, 704, 750, 757,
                    767, 768, 784, 789, 802, 807, 815, 818, 832, 834, 846, 854, 872, 873, 888, 894,
                    900, 908, 911, 930, 938, 950, 970, 973, 981, 985, 988, 993, 995, 998, 1000,
                    1002, 1005, 1006, 1012, 1016, 1018, 1019, 1022, 1024, 1031, 1032, 1034, 1035,
                    1037, 1040, 1044, 1045, 1046, 1048, 1049, 1050, 1051, 1052, 1054, 1058, 1062,
                    1063, 1064, 1066, 1067, 1068, 1069, 1070, 1072, 1073, 1075, 1076, 1077, 1079,
                    1080, 1082, 1083, 1084, 1087, 1088, 1089, 1090, 1092, 1093, 1094, 1095, 1096,
                    1097, 1099, 1101, 1102, 1103, 1104, 1106, 1107, 1108, 1109, 1110, 1111, 1112,
                    1113, 1114, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125, 1126, 1127,
                    1128, 1129, 1132, 1133, 1134, 1136, 1137, 1139, 1140, 1141, 1144, 1145, 1146,
                    1147, 1148, 1149, 1150, 1151, 1152, 1153, 1155, 1156, 1157, 1158, 1161, 1162,
                    1163, 1165, 1167, 1168, 1170, 1171, 1172, 1174, 1175, 1179, 1180, 1181, 1183,
                    1184, 1185, 1186, 1188, 1190, 1192, 1194, 1196, 1201, 1202, 1204, 1205, 1206,
                    1209, 1210, 1212, 1213, 1216, 1217, 1218, 1220, 1221, 1224, 1225, 1226, 1229,
                    1230, 1231, 1235, 1236, 1238, 1239, 1240, 1241, 1243, 1248, 1249, 1250, 1251,
                    1252, 1262, 1265, 1266, 1268, 1269, 1271, 1272, 1273, 1275, 1276, 1280, 1281,
                    1283, 1284, 1287, 1288, 1290, 1291, 1293, 1294, 1295, 1296, 1297, 1299, 1302,
                    1303, 1306, 1307, 1308, 1310, 1311, 1312, 1314, 1315, 1318, 1319, 1321, 1322,
                    1324, 1325, 1326, 1328, 1331, 1332, 1333, 1334, 1335, 1337, 1338, 1339, 1340,
                    1342, 1344, 1345, 1346, 1347, 1348, 1349, 1353, 1355, 1356, 1359, 1360, 1361,
                    1364, 1365, 1368, 1369, 1370, 1371, 1372, 1373, 1374, 1375, 1377, 1378, 1380,
                    1381, 1382, 1390, 1393, 1396, 1397, 1398, 1400, 1403, 1404, 1406, 1409, 1410,
                    1413, 1414, 1417, 1418, 1421, 1422, 1425, 1426, 1429, 1431, 1435, 1442, 1447,
                    1448, 1450, 1454, 1456, 1457, 1460, 1461, 1463, 1465, 1468, 1473, 1475, 1477,
                    1481, 1484, 1486, 1488, 1496, 1498, 1499, 1502, 1504, 1508, 1512, 1521, 1522,
                    1524, 1525, 1528, 1802, 2287, 2343, 2344, 2580, 3142, 5901, 5915, 5968, 5969,
                    5970, 5971, 5998, 5999, 6000, 6001, 6002, 6003, 6004, 6005, 6006, 6007, 6008,
                    6009, 6010, 6011, 6012, 6013, 6014, 6015, 6016, 6017, 6018, 6019, 6020, 6021,
                    6022, 6023, 6024, 6025, 6026, 6027, 6028, 6029, 6030, 6031, 6032, 6033, 6034,
                    6035, 6036, 6037, 6038, 6039, 6040, 6041, 6042, 6043, 6044, 6045, 6046, 6047,
                    6048, 6049, 6096, 6097, 6098, 6099, 6100, 6101, 6102, 6103, 6104, 6105, 6106,
                    6107, 6108, 6109, 6110, 6111, 6112, 6113, 6114, 6115, 6116, 6117, 6118, 6119,
                    6120, 6121, 6122, 6123, 6124, 6125, 6126, 6127, 6128, 6129, 6130, 6131, 6132,
                    6133, 6134, 6135, 6136, 6137, 6138, 6139, 6140, 6141, 6142, 6143, 6144, 6145,
                    6146, 6147, 6148, 6149, 6150, 6151, 6152, 6153, 6154, 6155, 6156, 6157, 6158,
                    6159, 6160, 6161, 6162, 6163, 6164, 6165, 6166, 6167, 6168, 6169, 6170, 6171,
                    6172, 6173, 6174, 6175, 6176, 6177, 6178, 6179, 6180, 6181, 6182, 6183, 6184,
                    6185, 6186, 6187, 6188, 6189, 6190, 6191, 6192, 6193, 6194, 6195, 6196, 6197,
                    6198, 6199, 6200, 6201, 6202, 6203, 6204, 6205, 6206, 6207, 6208, 6209, 6210,
                    6211, 6212, 6213, 6214, 6215, 6216, 6217, 6218, 6219, 6220, 6221, 6222, 6223,
                    6224, 6225, 6226, 6227, 6228, 6229, 6230, 6231, 6232, 6233, 6234, 6235, 6236,
                    6237, 6238, 6239, 6240, 6241, 6242, 6243, 6244, 6245, 6246, 6247, 6248, 6249,
                    6250, 6251, 6252, 6253, 6254, 6255, 6256, 6257, 6258, 6259, 6260, 6261, 6262,
                    6263, 6264, 6265, 6266, 6267, 6268, 6269, 6270, 6271, 6272, 6273, 6274, 6275,
                    6278, 6280, 6281, 6282, 6283, 6285, 6286, 6287, 6288, 6289, 6290, 6291, 6292,
                    7052, 8374, 8375, 8376, 8378, 8609, 8610, 8612, 8613, 8614, 8615, 8616, 8617,
                    8618, 8619, 8620, 8621, 8624, 8631, 8632, 8633, 8649, 8650, 8651, 8652, 8653,
                    8658, 8659, 8660, 8661, 8662, 8663, 8664, 8665, 8666, 8667, 8668, 8669, 8670,
                    8671, 8672, 8673, 8674, 8675, 8676, 8678, 8679, 8680, 8681, 8682, 8683, 8685,
                    8686, 8687, 8688, 8689, 8690, 8691, 8692, 8693, 8694, 8695, 8696, 8697, 8698,
                    8699, 8700, 8701, 8702, 8703, 8704, 8705, 8706, 8707, 8708, 8709, 8710, 8711,
                    8713, 8714, 8715, 8716, 8717, 8718, 8719, 8721, 8722, 8723, 8724, 8725, 8726,
                    8735, 8736, 8737, 8738, 12621
                ]
                .into_iter()
                .map(ProvinceId)
                .collect(),
                weather: Weather {
                    period: vec![
                        Period {
                            between: vec![
                                DayMonth::from_str("0.0").expect("invalid daymonth"),
                                DayMonth::from_str("30.0").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(14.0), Temperature(18.0)],
                            no_phenomenon: Weight(0.9),
                            rain_light: Weight(0.05),
                            rain_heavy: Weight(0.05),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.1").expect("invalid daymonth"),
                                DayMonth::from_str("27.1").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(15.0), Temperature(19.0)],
                            no_phenomenon: Weight(0.9),
                            rain_light: Weight(0.05),
                            rain_heavy: Weight(0.05),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.2").expect("invalid daymonth"),
                                DayMonth::from_str("30.2").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(19.0), Temperature(21.0)],
                            no_phenomenon: Weight(0.8),
                            rain_light: Weight(0.10),
                            rain_heavy: Weight(0.10),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.3").expect("invalid daymonth"),
                                DayMonth::from_str("29.3").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(20.0), Temperature(23.0)],
                            no_phenomenon: Weight(0.7),
                            rain_light: Weight(0.4),
                            rain_heavy: Weight(0.3),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.4").expect("invalid daymonth"),
                                DayMonth::from_str("30.4").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(20.0), Temperature(23.0)],
                            no_phenomenon: Weight(0.5),
                            rain_light: Weight(0.2),
                            rain_heavy: Weight(0.3),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.5").expect("invalid daymonth"),
                                DayMonth::from_str("29.5").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(20.0), Temperature(23.0)],
                            no_phenomenon: Weight(0.4),
                            rain_light: Weight(0.3),
                            rain_heavy: Weight(0.3),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.6").expect("invalid daymonth"),
                                DayMonth::from_str("30.6").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(17.0), Temperature(20.0)],
                            no_phenomenon: Weight(0.3),
                            rain_light: Weight(0.4),
                            rain_heavy: Weight(0.3),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.7").expect("invalid daymonth"),
                                DayMonth::from_str("30.7").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(17.0), Temperature(20.0)],
                            no_phenomenon: Weight(0.3),
                            rain_light: Weight(0.4),
                            rain_heavy: Weight(0.3),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.8").expect("invalid daymonth"),
                                DayMonth::from_str("29.8").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(17.0), Temperature(20.0)],
                            no_phenomenon: Weight(0.4),
                            rain_light: Weight(0.2),
                            rain_heavy: Weight(0.2),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.9").expect("invalid daymonth"),
                                DayMonth::from_str("30.9").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(14.0), Temperature(18.0)],
                            no_phenomenon: Weight(0.6),
                            rain_light: Weight(0.2),
                            rain_heavy: Weight(0.2),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.10").expect("invalid daymonth"),
                                DayMonth::from_str("29.10").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(12.0), Temperature(18.0)],
                            no_phenomenon: Weight(0.8),
                            rain_light: Weight(0.1),
                            rain_heavy: Weight(0.1),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("0.11").expect("invalid daymonth"),
                                DayMonth::from_str("30.11").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(12.0), Temperature(17.0)],
                            no_phenomenon: Weight(0.9),
                            rain_light: Weight(0.05),
                            rain_heavy: Weight(0.05),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(1.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                        Period {
                            between: vec![
                                DayMonth::from_str("4.11").expect("invalid daymonth"),
                                DayMonth::from_str("21.11").expect("invalid daymonth")
                            ],
                            temperature: vec![Temperature(-10.0), Temperature(35.0)],
                            no_phenomenon: Weight(1.5),
                            rain_light: Weight(0.25),
                            rain_heavy: Weight(0.1),
                            snow: Weight(0.0),
                            blizzard: Weight(0.0),
                            arctic_water: Weight(0.0),
                            mud: Weight(0.0),
                            sandstorm: Weight(0.0),
                            min_snow_level: SnowLevel(0.0)
                        },
                    ]
                }
            }
        );
    }

    #[test]
    fn it_reads_strategic_regions_from_a_directory() {
        let strategicregions_path = Path::new("./test/map/strategicregions");
        let strategicregions = StrategicRegions::from_dir(strategicregions_path)
            .expect("failed to read strategicregions");
        assert_eq!(strategicregions.strategic_regions.len(), 177);
        assert_eq!(
            strategicregions
                .strategic_regions
                .get(&StrategicRegionId(161))
                .expect("failed to get strategic region")
                .name,
            StrategicRegionName("GWW".to_owned())
        );
    }
}

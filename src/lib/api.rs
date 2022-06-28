    use anyhow::Result;
    use reqwest::blocking::get;
    use url::Url;

    pub fn get_data(url: Url) -> Result<String> {
        get(url)?.text().map_err(|e| e.into())
    }

    #[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Statistics {
        pub message: String,
        pub data: Data,
    }

    #[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub date: String,
        pub day: i64,
        pub resource: String,
        pub stats: Stats,
        pub increase: Increase,
    }

    #[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Stats {
        #[serde(rename = "personnel_units")]
        pub personnel_units: i64,
        pub tanks: i64,
        #[serde(rename = "armoured_fighting_vehicles")]
        pub armoured_fighting_vehicles: i64,
        #[serde(rename = "artillery_systems")]
        pub artillery_systems: i64,
        pub mlrs: i64,
        #[serde(rename = "aa_warfare_systems")]
        pub aa_warfare_systems: i64,
        pub planes: i64,
        pub helicopters: i64,
        #[serde(rename = "vehicles_fuel_tanks")]
        pub vehicles_fuel_tanks: i64,
        #[serde(rename = "warships_cutters")]
        pub warships_cutters: i64,
        #[serde(rename = "cruise_missiles")]
        pub cruise_missiles: i64,
        #[serde(rename = "uav_systems")]
        pub uav_systems: i64,
        #[serde(rename = "special_military_equip")]
        pub special_military_equip: i64,
        #[serde(rename = "atgm_srbm_systems")]
        pub atgm_srbm_systems: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Increase {
        #[serde(rename = "personnel_units")]
        pub personnel_units: i64,
        pub tanks: i64,
        #[serde(rename = "armoured_fighting_vehicles")]
        pub armoured_fighting_vehicles: i64,
        #[serde(rename = "artillery_systems")]
        pub artillery_systems: i64,
        pub mlrs: i64,
        #[serde(rename = "aa_warfare_systems")]
        pub aa_warfare_systems: i64,
        pub planes: i64,
        pub helicopters: i64,
        #[serde(rename = "vehicles_fuel_tanks")]
        pub vehicles_fuel_tanks: i64,
        #[serde(rename = "warships_cutters")]
        pub warships_cutters: i64,
        #[serde(rename = "cruise_missiles")]
        pub cruise_missiles: i64,
        #[serde(rename = "uav_systems")]
        pub uav_systems: i64,
        #[serde(rename = "special_military_equip")]
        pub special_military_equip: i64,
        #[serde(rename = "atgm_srbm_systems")]
        pub atgm_srbm_systems: i64,
    }
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Wttr {
    pub current_condition: Vec<CurrentCondition>,
    pub nearest_area: Vec<Area>,
    pub request: Vec<Request>,
    pub weather: Vec<Weather>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentCondition {
    #[serde(rename = "FeelsLikeC")]
    pub feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    pub feels_like_f: String,
    pub cloudcover: String,
    pub humidity: String,
    #[serde(rename = "lang_sv")]
    pub lang_sv: Vec<LanguageValue>,
    #[serde(rename = "localObsDateTime")]
    pub local_obs_date_time: String,
    #[serde(rename = "observation_time")]
    pub observation_time: String,
    #[serde(rename = "precipInches")]
    pub precip_inches: String,
    #[serde(rename = "precipMM")]
    pub precip_mm: String,
    pub pressure: String,
    #[serde(rename = "pressureInches")]
    pub pressure_inches: String,
    #[serde(rename = "temp_C")]
    pub temp_c: String,
    #[serde(rename = "temp_F")]
    pub temp_f: String,
    #[serde(rename = "uvIndex")]
    pub uv_index: String,
    pub visibility: String,
    #[serde(rename = "visibilityMiles")]
    pub visibility_miles: String,
    #[serde(rename = "weatherCode")]
    pub weather_code: String,
    #[serde(rename = "weatherDesc")]
    pub weather_desc: Vec<WeatherDescription>,
    #[serde(rename = "weatherIconUrl")]
    pub weather_icon_url: Vec<WeatherIconUrl>,
    #[serde(rename = "winddir16Point")]
    pub wind_dir_16_point: String,
    #[serde(rename = "winddirDegree")]
    pub wind_dir_degree: String,
    #[serde(rename = "windspeedKmph")]
    pub windspeed_kmph: String,
    #[serde(rename = "windspeedMiles")]
    pub windspeed_miles: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LanguageValue {
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeatherDescription {
    pub value: WeatherCondition,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeatherIconUrl {
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Area {
    #[serde(rename = "areaName")]
    pub area_name: Vec<Value>,
    pub country: Vec<Value>,
    pub latitude: String,
    pub longitude: String,
    pub population: String,
    pub region: Vec<Value>,
    #[serde(rename = "weatherUrl")]
    pub weather_url: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub query: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Weather {
    pub astronomy: Vec<Astronomy>,
    #[serde(rename = "avgtempC")]
    pub avg_temp_c: String,
    #[serde(rename = "avgtempF")]
    pub avg_temp_f: String,
    #[serde(rename = "date")]
    pub date: String,
    pub hourly: Vec<Hourly>,
    #[serde(rename = "maxtempC")]
    pub max_temp_c: String,
    #[serde(rename = "maxtempF")]
    pub max_temp_f: String,
    #[serde(rename = "mintempC")]
    pub min_temp_c: String,
    #[serde(rename = "mintempF")]
    pub min_temp_f: String,
    #[serde(rename = "sunHour")]
    pub sun_hour: String,
    #[serde(rename = "totalSnow_cm")]
    pub total_snow_cm: String,
    #[serde(rename = "uvIndex")]
    pub uv_index: String,
}

impl Weather {
    pub fn weather_report(&self) -> String {
        // Conditions from 06 and forward
        let conditions = self.hourly[2..]
            .iter()
            .map(|h| h.weather_desc[0].value.to_base_weather())
            .collect::<Vec<_>>();
        let unique_conditions: Vec<&BaseWeather> = conditions.iter().unique().collect();

        let mut weather_counts: Vec<_> = unique_conditions
            .iter()
            .map(|&condition| {
                (
                    condition,
                    conditions.iter().filter(|&c| c == condition).count(),
                )
            })
            .collect();

        weather_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

        let mut report = String::new();

        for (i, &(condition, _)) in weather_counts.iter().enumerate() {
            let weather = condition.to_swedish();
            let separator = if i == 0 { "" } else { " och" };

            use BaseWeather::*;
            match condition {
                Clear | Sunny => {
                    if i == 0 {
                        report.push_str(&format!("Det blir {}", weather));
                    } else {
                        report.push_str(&format!("{} {}", separator, weather));
                    }
                }
                Cloudy => {
                    if i == 0 {
                        report.push_str("Det blir molnigt");
                    } else {
                        report.push_str(&format!("{} lite {}", separator, weather));
                    }
                }
                Rain => {
                    if i == 0 {
                        report.push_str(&format!("Imorgon blir det mest {}", weather));
                    } else {
                        report.push_str(&format!("{} {}", separator, weather));
                    }
                }
                Fog => {
                    if i == 0 {
                        report.push_str(&format!("Det kan bli lite {}", weather));
                    } else {
                        report.push_str(&format!("{} lite", separator));
                    }
                    report.push_str(&format!(" {}", weather));
                }
                // TODO: Fill in rest of the weather conditions
                _ => {
                    if i == 0 {
                        report.push_str(&format!(
                            "Förvänta dig {} under dagen, tillsammans med",
                            weather
                        ));
                    } else {
                        report.push_str(&format!("{} {}", separator, weather));
                    }
                }
            }
        }
        report.push('.');
        report
    }
}

impl ToString for Weather {
    fn to_string(&self) -> String {
        format!(
            "{}\nTemp min/max: {}/{}°C\nSoltimmar: {}\nUV-index {}: {}",
            // self.weather_condition_for_the_day(),
            self.weather_report(),
            self.min_temp_c,
            self.max_temp_c,
            self.sun_hour,
            self.uv_index,
            explain_uv_index(self.uv_index.parse().unwrap_or_default())
        )
    }
}

fn explain_uv_index(uv_index: u8) -> &'static str {
    match uv_index {
        0..=2 => "Lågt UV-index. Minimalt solskydd krävs.",
        3..=5 => "Måttligt UV-index. Sök skugga under middagstimmen, använd solskyddsmedel.",
        6..=7 => "Högt UV-index. Minska solexponeringen mellan kl. 10 och 16, använd solskyddsmedel med SPF 30+.",
        8..=10 => "Mycket högt UV-index. Ta extra försiktighet. Minimera solexponeringen.",
        _ => "Extremt UV-index. Undvik att vara utomhus. Solskydd är nödvändigt.",
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Astronomy {
    pub moon_illumination: String,
    pub moon_phase: String,
    pub moonrise: String,
    pub moonset: String,
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hourly {
    #[serde(rename = "DewPointC")]
    pub dew_point_c: String,
    #[serde(rename = "DewPointF")]
    pub dew_point_f: String,
    #[serde(rename = "FeelsLikeC")]
    pub feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    pub feels_like_f: String,
    #[serde(rename = "HeatIndexC")]
    pub heat_index_c: String,
    #[serde(rename = "HeatIndexF")]
    pub heat_index_f: String,
    #[serde(rename = "WindChillC")]
    pub wind_chill_c: String,
    #[serde(rename = "WindChillF")]
    pub wind_chill_f: String,
    #[serde(rename = "WindGustKmph")]
    pub wind_gust_kmph: String,
    #[serde(rename = "WindGustMiles")]
    pub wind_gust_miles: String,
    #[serde(rename = "chanceoffog")]
    pub chance_of_fog: String,
    #[serde(rename = "chanceoffrost")]
    pub chance_of_frost: String,
    #[serde(rename = "chanceofhightemp")]
    pub chance_of_high_temp: String,
    #[serde(rename = "chanceofovercast")]
    pub chance_of_overcast: String,
    #[serde(rename = "chanceofrain")]
    pub chance_of_rain: String,
    #[serde(rename = "chanceofremdry")]
    pub chance_of_rem_dry: String,
    #[serde(rename = "chanceofsnow")]
    pub chance_of_snow: String,
    #[serde(rename = "chanceofsunshine")]
    pub chance_of_sunshine: String,
    #[serde(rename = "chanceofthunder")]
    pub chance_of_thunder: String,
    #[serde(rename = "chanceofwindy")]
    pub chance_of_windy: String,
    #[serde(rename = "cloudcover")]
    pub cloud_cover: String,
    #[serde(rename = "humidity")]
    pub humidity: String,
    #[serde(rename = "lang_sv")]
    pub lang_sv: Vec<LanguageValue>,
    #[serde(rename = "precipInches")]
    pub precip_inches: String,
    #[serde(rename = "precipMM")]
    pub precip_mm: String,
    #[serde(rename = "pressure")]
    pub pressure: String,
    #[serde(rename = "pressureInches")]
    pub pressure_inches: String,
    #[serde(rename = "tempC")]
    pub temp_c: String,
    #[serde(rename = "tempF")]
    pub temp_f: String,
    #[serde(rename = "time")]
    pub time: String,
    #[serde(rename = "uvIndex")]
    pub uv_index: String,
    #[serde(rename = "visibility")]
    pub visibility: String,
    #[serde(rename = "visibilityMiles")]
    pub visibility_miles: String,
    #[serde(rename = "weatherCode")]
    pub weather_code: String,
    #[serde(rename = "weatherDesc")]
    pub weather_desc: Vec<WeatherDescription>,
    #[serde(rename = "weatherIconUrl")]
    pub weather_icon_url: Vec<WeatherIconUrl>,
    #[serde(rename = "winddir16Point")]
    pub wind_dir_16_point: String,
    #[serde(rename = "winddirDegree")]
    pub wind_dir_degree: String,
    #[serde(rename = "windspeedKmph")]
    pub windspeed_kmph: String,
    #[serde(rename = "windspeedMiles")]
    pub windspeed_miles: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum WeatherCondition {
    #[serde(rename = "Clear")]
    Clear,
    #[serde(rename = "Sunny")]
    Sunny,
    #[serde(rename = "Partly cloudy")]
    PartlyCloudy,
    #[serde(rename = "Cloudy")]
    Cloudy,
    #[serde(rename = "Overcast")]
    Overcast,
    #[serde(rename = "Mist")]
    Mist,
    #[serde(rename = "Patchy rain possible")]
    PatchyRainPossible,
    #[serde(rename = "Patchy snow possible")]
    PatchySnowPossible,
    #[serde(rename = "Patchy sleet possible")]
    PatchySleetPossible,
    #[serde(rename = "Patchy freezing drizzle possible")]
    PatchyFreezingDrizzlePossible,
    #[serde(rename = "Thundery outbreaks possible")]
    ThunderyOutbreaksPossible,
    #[serde(rename = "Blowing snow")]
    BlowingSnow,
    #[serde(rename = "Blizzard")]
    Blizzard,
    #[serde(rename = "Fog")]
    Fog,
    #[serde(rename = "Freezing fog")]
    FreezingFog,
    #[serde(rename = "Patchy light drizzle")]
    PatchyLightDrizzle,
    #[serde(rename = "Light drizzle")]
    LightDrizzle,
    #[serde(rename = "Freezing drizzle")]
    FreezingDrizzle,
    #[serde(rename = "Heavy freezing drizzle")]
    HeavyFreezingDrizzle,
    #[serde(rename = "Patchy light rain")]
    PatchyLightRain,
    #[serde(rename = "Light rain")]
    LightRain,
    #[serde(rename = "Moderate rain at times")]
    ModerateRainAtTimes,
    #[serde(rename = "Moderate rain")]
    ModerateRain,
    #[serde(rename = "Heavy rain at times")]
    HeavyRainAtTimes,
    #[serde(rename = "Heavy rain")]
    HeavyRain,
    #[serde(rename = "Light freezing rain")]
    LightFreezingRain,
    #[serde(rename = "Moderate or heavy freezing rain")]
    ModerateOrHeavyFreezingRain,
    #[serde(rename = "Light sleet")]
    LightSleet,
    #[serde(rename = "Moderate or heavy sleet")]
    ModerateOrHeavySleet,
    #[serde(rename = "Patchy light snow")]
    PatchyLightSnow,
    #[serde(rename = "Light snow")]
    LightSnow,
    #[serde(rename = "Patchy moderate snow")]
    PatchyModerateSnow,
    #[serde(rename = "Moderate snow")]
    ModerateSnow,
    #[serde(rename = "Patchy heavy snow")]
    PatchyHeavySnow,
    #[serde(rename = "Heavy snow")]
    HeavySnow,
    #[serde(rename = "Ice pellets")]
    IcePellets,
    #[serde(rename = "Light rain shower")]
    LightRainShower,
    #[serde(rename = "Moderate or heavy rain shower")]
    ModerateOrHeavyRainShower,
    #[serde(rename = "Torrential rain shower")]
    TorrentialRainShower,
    #[serde(rename = "Light sleet showers")]
    LightSleetShowers,
    #[serde(rename = "Moderate or heavy sleet showers")]
    ModerateOrHeavySleetShowers,
    #[serde(rename = "Light snow showers")]
    LightSnowShowers,
    #[serde(rename = "Moderate or heavy snow showers")]
    ModerateOrHeavySnowShowers,
    #[serde(rename = "Patchy light rain with thunder")]
    PatchyLightRainWithThunder,
    #[serde(rename = "Moderate or heavy rain with thunder")]
    ModerateOrHeavyRainWithThunder,
    #[serde(rename = "Patchy light snow with thunder")]
    PatchyLightSnowWithThunder,
    #[serde(rename = "Moderate or heavy snow with thunder")]
    ModerateOrHeavySnowWithThunder,
    // Simplified, not part of the wttr API
    Rainy,
    Snowy,
    Sleet,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum BaseWeather {
    Rain,
    Clear,
    Cloudy,
    Sunny,
    Snow,
    Sleet,
    Fog,
    Thunder,
}

impl BaseWeather {
    pub fn to_swedish(&self) -> String {
        let s = match self {
            BaseWeather::Rain => "regn",
            BaseWeather::Clear => "klar himmel",
            BaseWeather::Cloudy => "moln",
            BaseWeather::Sunny => "sol",
            BaseWeather::Snow => "snö",
            BaseWeather::Sleet => "snöblandat regn",
            BaseWeather::Fog => "dimma",
            BaseWeather::Thunder => "åska",
        };
        s.to_string()
    }
}

impl WeatherCondition {
    #[rustfmt::skip]
    fn to_base_weather(&self) -> BaseWeather {
        match *self {
            WeatherCondition::PatchySnowPossible
            | WeatherCondition::PatchyFreezingDrizzlePossible
            | WeatherCondition::BlowingSnow
            | WeatherCondition::Blizzard
            | WeatherCondition::PatchyLightSnow
            | WeatherCondition::LightSnow
            | WeatherCondition::PatchyModerateSnow
            | WeatherCondition::ModerateSnow
            | WeatherCondition::PatchyHeavySnow
            | WeatherCondition::HeavySnow
            | WeatherCondition::IcePellets
            | WeatherCondition::LightSnowShowers
            | WeatherCondition::ModerateOrHeavySnowShowers
            | WeatherCondition::PatchyLightSnowWithThunder
            | WeatherCondition::Snowy
            | WeatherCondition::ModerateOrHeavySnowWithThunder => BaseWeather::Snow,

            WeatherCondition::PatchyRainPossible
            | WeatherCondition::LightDrizzle
            | WeatherCondition::FreezingDrizzle
            | WeatherCondition::HeavyFreezingDrizzle
            | WeatherCondition::PatchyLightRain
            | WeatherCondition::LightRain
            | WeatherCondition::ModerateRainAtTimes
            | WeatherCondition::ModerateRain
            | WeatherCondition::HeavyRainAtTimes
            | WeatherCondition::HeavyRain
            | WeatherCondition::LightFreezingRain
            | WeatherCondition::ModerateOrHeavyFreezingRain
            | WeatherCondition::LightRainShower
            | WeatherCondition::ModerateOrHeavyRainShower
            | WeatherCondition::TorrentialRainShower
            | WeatherCondition::PatchyLightRainWithThunder
            | WeatherCondition::PatchyLightDrizzle
            | WeatherCondition::ModerateOrHeavyRainWithThunder
            | WeatherCondition::Rainy => BaseWeather::Rain,

            WeatherCondition::Mist
            | WeatherCondition::Fog
            | WeatherCondition::FreezingFog => BaseWeather::Fog,

            WeatherCondition::Clear => BaseWeather::Clear,

            WeatherCondition::Cloudy
            | WeatherCondition::Overcast
            | WeatherCondition::PartlyCloudy => BaseWeather::Cloudy,

            WeatherCondition::ThunderyOutbreaksPossible => BaseWeather::Thunder,

            WeatherCondition::LightSleet
            | WeatherCondition::ModerateOrHeavySleet
            | WeatherCondition::LightSleetShowers
            | WeatherCondition::PatchySleetPossible
            | WeatherCondition::ModerateOrHeavySleetShowers
            | WeatherCondition::Sleet => BaseWeather::Sleet,

            WeatherCondition::Sunny => BaseWeather::Sunny,
        }
    }
}

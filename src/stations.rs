use super::Pandora;
use super::error::Result;
use super::method::Method;
use super::music::{ToMusicToken, MusicType};
use super::playlist::Playlist;

use serde_json;

pub struct Stations<'a> {
    pandora: &'a Pandora,
}

impl<'a> Stations<'a> {
    pub fn new(pandora: &'a Pandora) -> Stations<'a> {
        Stations { pandora: pandora }
    }

    pub fn list(&self) -> Result<Vec<Station>> {
        let stations = self.pandora.request::<StationList>(Method::UserGetStationList, None)?;
        Ok(stations.stations)
    }

    pub fn create<T>(&self, music_token: &T) -> Result<Station> where T: ToMusicToken {
        self.pandora.request(Method::StationCreateStation, Some(serde_json::to_value(CreateStationRequest {
                                                                                    track_token: None,
                                                                                    music_type: None,
                                                                                    music_token: Some(music_token.to_music_token()),
                                                                                }).unwrap()))
    }

    pub fn rename<T>(&self, station: &T, station_name: &str) -> Result<Station> where T: ToStationToken {
        self.pandora.request(Method::StationRenameStation, Some(serde_json::to_value(RenameStationRequest {
                                                                                    station_token: station.to_station_token(),
                                                                                    station_name: station_name.to_owned(),
                                                                                }).unwrap()))
    }

    pub fn delete<T>(&self, station: &T) -> Result<()> where T: ToStationToken {
        self.pandora.request_noop(Method::StationDeleteStation, Some(serde_json::to_value(DeleteStationRequest {
                                                                                    station_token: station.to_station_token(),
                                                                                }).unwrap()))
    }

    pub fn add_seed<S, T>(&self, station: &S, music_token: &T) -> Result<Seed> where S: ToStationToken, T: ToMusicToken {
        self.pandora.request(Method::StationAddMusic, Some(serde_json::to_value(AddSeedRequest {
                                                                                    station_token: station.to_station_token(),
                                                                                    music_token: music_token.to_music_token(),
                                                                                }).unwrap()))
    }

    pub fn remove_seed(&self, seed: &Seed) -> Result<()> {
        self.pandora.request(Method::StationDeleteMusic, Some(serde_json::to_value(RemoveSeedRequest { 
                                                                                    seed_id: seed.seed_id.clone() 
                                                                                }).unwrap()))
    }

    pub fn station<T>(&self, station: &T) -> Result<Station> where T: ToStationToken {
        self.pandora.request(Method::StationGetStation, Some(serde_json::to_value(GetStationRequest {
                                                                                    station_token: station.to_station_token(),
                                                                                    include_extended_attributes: true,
                                                                                }).unwrap()))
    }

    // Gets the current checksum of the station; useful if you need to check for changes.
    pub fn checksum(&self) -> Result<StationListChecksum> {
        self.pandora.request(Method::UserGetStationListChecksum, None)
    }

    pub fn playlist<T>(&self, station: &T) -> Playlist where T: ToStationToken {
        Playlist::new(self.pandora, station)
    }
}

pub trait ToStationToken {
    fn to_station_token(&self) -> String;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Station {
    #[serde(rename="stationId")]
    pub station_id: String,
    #[serde(rename="stationName")]
    pub station_name: String,
}

impl ToStationToken for Station {
    fn to_station_token(&self) -> String {
        self.station_id.clone()
    }
}

#[derive(Debug, Deserialize)]
struct StationList {
    pub stations: Vec<Station>,
    pub checksum: String,
}

#[derive(Deserialize)]
pub struct StationListChecksum {
    pub checksum: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtendedStation {
    #[serde(rename="stationId")]
    pub station_id: String,
    #[serde(rename="stationName")]
    pub station_name: String,
    #[serde(rename="artUrl")]
    pub art_url: Option<String>,
    // Some stations don't allow adding music (e.g. QuickMix).
    pub music: Option<StationMusic>,
}

/// Seed information for a station.
#[derive(Debug, Deserialize)]
pub struct StationMusic {
    pub songs: Vec<SongSeed>,
    pub artists: Vec<ArtistSeed>,
    pub genre: Option<Vec<GenreSeed>>,
}

/// Generic seed.
#[derive(Debug, Deserialize)]
pub struct Seed {
    #[serde(rename="seedId")]
    pub seed_id: String,
}

/// Song seed.
#[derive(Debug, Deserialize)]
pub struct SongSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="artUrl")]
    pub art_url: String,
    #[serde(rename="songName")]
    pub song_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

/// Artist seed.
#[derive(Debug, Deserialize)]
pub struct ArtistSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub artist_name: String,
    #[serde(rename="artUrl")]
    pub art_url: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

/// Genre seed.
#[derive(Debug, Deserialize)]
pub struct GenreSeed {
    #[serde(rename="seedId")]
    pub seed_id: String,
    #[serde(rename="artistName")]
    pub genre_name: String,
    #[serde(rename="musicToken")]
    pub music_token: String,
}

////////////////////
// Request structs
////////////////////

#[derive(Serialize)]
struct CreateStationRequest {
    #[serde(rename="trackToken")]
    track_token: Option<String>,
    #[serde(rename="musicType")]
    music_type: Option<MusicType>,
    #[serde(rename="musicToken")]
    music_token: Option<String>,
}

#[derive(Serialize)]
struct RenameStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="stationName")]
    station_name: String,
}

#[derive(Serialize)]
struct DeleteStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
}

#[derive(Serialize)]
struct GetStationRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="includeExtendedAttributes")]
    include_extended_attributes: bool,
}

#[derive(Serialize)]
struct AddSeedRequest {
    #[serde(rename="stationToken")]
    station_token: String,
    #[serde(rename="musicToken")]
    music_token: String,
}

#[derive(Serialize)]
struct RemoveSeedRequest {
    #[serde(rename="seedId")]
    seed_id: String,
}

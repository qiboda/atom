
/*!
<auto-generated>
    This code was generated by a tool.
    Changes to this file may cause incorrect behavior and will be lost if
    the code is regenerated.
</auto-generated>
*/


use super::*;

///camp relationship
#[derive(Debug, Hash, Eq, PartialEq, bevy::reflect::Reflect, macros::EnumFromNum)]
pub enum RelationShipType {
    None = 0,
    Hostility = 1,
    Friendly = 2,
}

impl From<i32> for RelationShipType {
    fn from(value: i32) -> Self {
        match value { 
            0 => RelationShipType::None,
            1 => RelationShipType::Hostility,
            2 => RelationShipType::Friendly,
            _ => panic!("Invalid value for RelationShipType:{}", value),
        }
    }
}

#[derive(bevy::reflect::Reflect, Debug)]
pub struct Monster {
    /// 这是id
    pub id: i32,
    /// 名字
    pub name: String,
    /// 描述
    pub desc: String,
    /// 阵营
    pub camp: i32,
}

impl Monster{
    pub fn new(json: &serde_json::Value) -> Result<Monster, LubanError> {
        let id = (json["id"].as_i64().unwrap() as i32);
        let name = json["name"].as_str().unwrap().to_string();
        let desc = json["desc"].as_str().unwrap().to_string();
        let camp = (json["camp"].as_i64().unwrap() as i32);
        
        Ok(Monster { id, name, desc, camp, })
    }
}

#[derive(bevy::reflect::Reflect, Debug)]
pub struct Npc {
    /// 这是id
    pub id: i32,
    /// 名字
    pub name: String,
    /// 描述
    pub desc: String,
    /// 阵营
    pub camp: i32,
}

impl Npc{
    pub fn new(json: &serde_json::Value) -> Result<Npc, LubanError> {
        let id = (json["id"].as_i64().unwrap() as i32);
        let name = json["name"].as_str().unwrap().to_string();
        let desc = json["desc"].as_str().unwrap().to_string();
        let camp = (json["camp"].as_i64().unwrap() as i32);
        
        Ok(Npc { id, name, desc, camp, })
    }
}

#[derive(bevy::reflect::Reflect, Debug)]
pub struct Player {
    /// 这是id
    pub id: i32,
    /// 名字
    pub name: String,
    /// 描述
    pub desc: String,
    /// 阵营
    pub camp: i32,
    /// 碰撞体半径
    pub capsule_radius: f32,
    /// 碰撞体高度
    pub capsule_height: f32,
}

impl Player{
    pub fn new(json: &serde_json::Value) -> Result<Player, LubanError> {
        let id = (json["id"].as_i64().unwrap() as i32);
        let name = json["name"].as_str().unwrap().to_string();
        let desc = json["desc"].as_str().unwrap().to_string();
        let camp = (json["camp"].as_i64().unwrap() as i32);
        let capsule_radius = (json["capsule_radius"].as_f64().unwrap() as f32);
        let capsule_height = (json["capsule_height"].as_f64().unwrap() as f32);
        
        Ok(Player { id, name, desc, camp, capsule_radius, capsule_height, })
    }
}

#[derive(bevy::reflect::Reflect, Debug)]
pub struct RelationShip {
    /// 主动方阵营
    pub active_camp: i32,
    /// 被动方阵营
    pub passive_camp: i32,
    /// 关系
    pub relationship_type: crate::unit::RelationShipType,
}

impl RelationShip{
    pub fn new(json: &serde_json::Value) -> Result<RelationShip, LubanError> {
        let active_camp = (json["active_camp"].as_i64().unwrap() as i32);
        let passive_camp = (json["passive_camp"].as_i64().unwrap() as i32);
        let relationship_type = json["relationship_type"].as_i64().unwrap().into();
        
        Ok(RelationShip { active_camp, passive_camp, relationship_type, })
    }
}


#[derive(Debug, bevy::reflect::Reflect, bevy::asset::Asset)]
pub struct TbMonster {
    pub data_list: Vec<std::sync::Arc<crate::unit::Monster>>,
    pub data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Monster>>,
}

impl TbMonster {
    pub fn new(json: &serde_json::Value) -> Result<TbMonster, LubanError> {
        let mut data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Monster>> = Default::default();
        let mut data_list: Vec<std::sync::Arc<crate::unit::Monster>> = vec![];

        for x in json.as_array().unwrap() {
            let row = std::sync::Arc::new(crate::unit::Monster::new(&x)?);
            data_list.push(row.clone());
            data_map.insert(row.id.clone(), row.clone());
        }

        Ok(TbMonster { data_map, data_list })
    }

    pub fn get(&self, key: &i32) -> Option<std::sync::Arc<crate::unit::Monster>> {
        self.data_map.get(key).map(|x| x.clone())
    }
}

impl std::ops::Index<i32> for TbMonster {
    type Output = std::sync::Arc<crate::unit::Monster>;

    fn index(&self, index: i32) -> &Self::Output {
        &self.data_map.get(&index).unwrap()
    }
}
impl luban_lib::table::Table for TbMonster {
    type Value = std::sync::Arc<crate::unit::Monster>;
}
pub type TbMonsterKey = i32;
#[derive(Debug, Default, Clone, bevy::reflect::Reflect, bevy::prelude::Component, serde::Serialize, serde::Deserialize)]
pub struct TbMonsterRow {
    pub key: TbMonsterKey,
    #[serde(skip)]
    pub data: Option<std::sync::Arc<crate::unit::Monster>>,
}

impl PartialEq for TbMonsterRow {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl TbMonsterRow {
    pub fn new(key: TbMonsterKey, data: Option<std::sync::Arc<crate::unit::Monster>>) -> Self {
        Self { key, data }
    }

    pub fn key(&self) -> &TbMonsterKey {
        &self.key
    }

    pub fn set_key(&mut self, key: TbMonsterKey) {
        self.key = key;
    }

    pub fn set_data(&mut self, data: Option<std::sync::Arc<crate::unit::Monster>>) {
        self.data = data;
    }

    pub fn get_data(&self) -> Option<std::sync::Arc<crate::unit::Monster>> {
        self.data.clone()
    }

    pub fn data(&self) -> std::sync::Arc<crate::unit::Monster> {
        self.data.clone().unwrap()
    }
}


impl luban_lib::table::MapTable for TbMonster {
    type Key = TbMonsterKey;
    type List = Vec<std::sync::Arc<crate::unit::Monster>>;
    type Map = bevy::utils::HashMap<Self::Key, Self::Value>;

    fn get_row(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data_map.get(key).map(|x| x.clone())
    }

    fn get_data_list(&self) -> &Self::List {
        &self.data_list
    }

    fn get_data_map(&self) -> &Self::Map {
        &self.data_map
    }
}


#[derive(Debug, Default)]
pub struct TbMonsterLoader;

impl bevy::asset::AssetLoader for TbMonsterLoader {
    type Asset = TbMonster;

    type Settings = ();

    type Error = TableLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::info!("TbMonsterLoader loading start");
        let mut bytes = Vec::new();
        use bevy::asset::AsyncReadExt;
        reader.read_to_end(&mut bytes).await?;
        let t = serde_json::from_slice::<serde_json::Value>(&bytes)?;
        let tb = TbMonster::new(&t).unwrap();
        bevy::log::info!("TbMonsterLoader loading over");
        Ok(tb)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}


#[derive(Debug, bevy::reflect::Reflect, bevy::asset::Asset)]
pub struct TbNpc {
    pub data_list: Vec<std::sync::Arc<crate::unit::Npc>>,
    pub data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Npc>>,
}

impl TbNpc {
    pub fn new(json: &serde_json::Value) -> Result<TbNpc, LubanError> {
        let mut data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Npc>> = Default::default();
        let mut data_list: Vec<std::sync::Arc<crate::unit::Npc>> = vec![];

        for x in json.as_array().unwrap() {
            let row = std::sync::Arc::new(crate::unit::Npc::new(&x)?);
            data_list.push(row.clone());
            data_map.insert(row.id.clone(), row.clone());
        }

        Ok(TbNpc { data_map, data_list })
    }

    pub fn get(&self, key: &i32) -> Option<std::sync::Arc<crate::unit::Npc>> {
        self.data_map.get(key).map(|x| x.clone())
    }
}

impl std::ops::Index<i32> for TbNpc {
    type Output = std::sync::Arc<crate::unit::Npc>;

    fn index(&self, index: i32) -> &Self::Output {
        &self.data_map.get(&index).unwrap()
    }
}
impl luban_lib::table::Table for TbNpc {
    type Value = std::sync::Arc<crate::unit::Npc>;
}
pub type TbNpcKey = i32;
#[derive(Debug, Default, Clone, bevy::reflect::Reflect, bevy::prelude::Component, serde::Serialize, serde::Deserialize)]
pub struct TbNpcRow {
    pub key: TbNpcKey,
    #[serde(skip)]
    pub data: Option<std::sync::Arc<crate::unit::Npc>>,
}

impl PartialEq for TbNpcRow {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl TbNpcRow {
    pub fn new(key: TbNpcKey, data: Option<std::sync::Arc<crate::unit::Npc>>) -> Self {
        Self { key, data }
    }

    pub fn key(&self) -> &TbNpcKey {
        &self.key
    }

    pub fn set_key(&mut self, key: TbNpcKey) {
        self.key = key;
    }

    pub fn set_data(&mut self, data: Option<std::sync::Arc<crate::unit::Npc>>) {
        self.data = data;
    }

    pub fn get_data(&self) -> Option<std::sync::Arc<crate::unit::Npc>> {
        self.data.clone()
    }

    pub fn data(&self) -> std::sync::Arc<crate::unit::Npc> {
        self.data.clone().unwrap()
    }
}


impl luban_lib::table::MapTable for TbNpc {
    type Key = TbNpcKey;
    type List = Vec<std::sync::Arc<crate::unit::Npc>>;
    type Map = bevy::utils::HashMap<Self::Key, Self::Value>;

    fn get_row(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data_map.get(key).map(|x| x.clone())
    }

    fn get_data_list(&self) -> &Self::List {
        &self.data_list
    }

    fn get_data_map(&self) -> &Self::Map {
        &self.data_map
    }
}


#[derive(Debug, Default)]
pub struct TbNpcLoader;

impl bevy::asset::AssetLoader for TbNpcLoader {
    type Asset = TbNpc;

    type Settings = ();

    type Error = TableLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::info!("TbNpcLoader loading start");
        let mut bytes = Vec::new();
        use bevy::asset::AsyncReadExt;
        reader.read_to_end(&mut bytes).await?;
        let t = serde_json::from_slice::<serde_json::Value>(&bytes)?;
        let tb = TbNpc::new(&t).unwrap();
        bevy::log::info!("TbNpcLoader loading over");
        Ok(tb)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}


#[derive(Debug, bevy::reflect::Reflect, bevy::asset::Asset)]
pub struct TbPlayer {
    pub data_list: Vec<std::sync::Arc<crate::unit::Player>>,
    pub data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Player>>,
}

impl TbPlayer {
    pub fn new(json: &serde_json::Value) -> Result<TbPlayer, LubanError> {
        let mut data_map: bevy::utils::HashMap<i32, std::sync::Arc<crate::unit::Player>> = Default::default();
        let mut data_list: Vec<std::sync::Arc<crate::unit::Player>> = vec![];

        for x in json.as_array().unwrap() {
            let row = std::sync::Arc::new(crate::unit::Player::new(&x)?);
            data_list.push(row.clone());
            data_map.insert(row.id.clone(), row.clone());
        }

        Ok(TbPlayer { data_map, data_list })
    }

    pub fn get(&self, key: &i32) -> Option<std::sync::Arc<crate::unit::Player>> {
        self.data_map.get(key).map(|x| x.clone())
    }
}

impl std::ops::Index<i32> for TbPlayer {
    type Output = std::sync::Arc<crate::unit::Player>;

    fn index(&self, index: i32) -> &Self::Output {
        &self.data_map.get(&index).unwrap()
    }
}
impl luban_lib::table::Table for TbPlayer {
    type Value = std::sync::Arc<crate::unit::Player>;
}
pub type TbPlayerKey = i32;
#[derive(Debug, Default, Clone, bevy::reflect::Reflect, bevy::prelude::Component, serde::Serialize, serde::Deserialize)]
pub struct TbPlayerRow {
    pub key: TbPlayerKey,
    #[serde(skip)]
    pub data: Option<std::sync::Arc<crate::unit::Player>>,
}

impl PartialEq for TbPlayerRow {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl TbPlayerRow {
    pub fn new(key: TbPlayerKey, data: Option<std::sync::Arc<crate::unit::Player>>) -> Self {
        Self { key, data }
    }

    pub fn key(&self) -> &TbPlayerKey {
        &self.key
    }

    pub fn set_key(&mut self, key: TbPlayerKey) {
        self.key = key;
    }

    pub fn set_data(&mut self, data: Option<std::sync::Arc<crate::unit::Player>>) {
        self.data = data;
    }

    pub fn get_data(&self) -> Option<std::sync::Arc<crate::unit::Player>> {
        self.data.clone()
    }

    pub fn data(&self) -> std::sync::Arc<crate::unit::Player> {
        self.data.clone().unwrap()
    }
}


impl luban_lib::table::MapTable for TbPlayer {
    type Key = TbPlayerKey;
    type List = Vec<std::sync::Arc<crate::unit::Player>>;
    type Map = bevy::utils::HashMap<Self::Key, Self::Value>;

    fn get_row(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data_map.get(key).map(|x| x.clone())
    }

    fn get_data_list(&self) -> &Self::List {
        &self.data_list
    }

    fn get_data_map(&self) -> &Self::Map {
        &self.data_map
    }
}


#[derive(Debug, Default)]
pub struct TbPlayerLoader;

impl bevy::asset::AssetLoader for TbPlayerLoader {
    type Asset = TbPlayer;

    type Settings = ();

    type Error = TableLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::info!("TbPlayerLoader loading start");
        let mut bytes = Vec::new();
        use bevy::asset::AsyncReadExt;
        reader.read_to_end(&mut bytes).await?;
        let t = serde_json::from_slice::<serde_json::Value>(&bytes)?;
        let tb = TbPlayer::new(&t).unwrap();
        bevy::log::info!("TbPlayerLoader loading over");
        Ok(tb)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}


#[derive(Debug, bevy::reflect::Reflect, bevy::asset::Asset)]
pub struct TbRelationShip {
    pub data_list: Vec<std::sync::Arc<crate::unit::RelationShip>>,
    pub data_map_union: bevy::utils::HashMap<(i32, i32), std::sync::Arc<crate::unit::RelationShip>>,
}

impl TbRelationShip {
    pub fn new(json: &serde_json::Value) -> Result<TbRelationShip, LubanError> {
        let mut data_list: Vec<std::sync::Arc<crate::unit::RelationShip>> = vec![];

        for x in json.as_array().unwrap() {
            let row = std::sync::Arc::new(crate::unit::RelationShip::new(&x)?);
            data_list.push(row.clone());
        }
        let mut data_map_union: bevy::utils::HashMap<(i32, i32), std::sync::Arc<crate::unit::RelationShip>> = Default::default();
        for x in &data_list {
            data_map_union.insert((x.active_camp, x.passive_camp.clone()), x.clone());
        }

    Ok(TbRelationShip { 
            data_list,
            data_map_union,
        })
    }

    pub fn get(&self, key: &(i32, i32)) -> Option<std::sync::Arc<crate::unit::RelationShip>> {
        self.data_map_union.get(key).map(|x| x.clone())
    }
}

impl luban_lib::table::Table for TbRelationShip {
    type Value = std::sync::Arc<crate::unit::RelationShip>;
}
impl luban_lib::table::ListTable for TbRelationShip {}


pub type TbRelationShipKey = (i32, i32);
#[derive(Debug, Default, Clone, bevy::reflect::Reflect, bevy::prelude::Component, serde::Serialize, serde::Deserialize)]
pub struct TbRelationShipRow {
    pub key: TbRelationShipKey,
    #[serde(skip)]
    pub data: Option<std::sync::Arc<crate::unit::RelationShip>>,
}

impl PartialEq for TbRelationShipRow {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl luban_lib::table::MultiUnionIndexListTable for TbRelationShip {
    type Key = TbRelationShipKey;
    type List = Vec<std::sync::Arc<crate::unit::RelationShip>>;
    type Map = bevy::utils::HashMap<Self::Key, Self::Value>;

    fn get_row_by_key(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data_map_union.get(key).map(|x| x.clone())
    }

    fn get_data_list(&self) -> &Self::List {
        &self.data_list
    }

    fn get_data_map(&self) -> &Self::Map {
        &self.data_map_union
    }
}


#[derive(Debug, Default)]
pub struct TbRelationShipLoader;

impl bevy::asset::AssetLoader for TbRelationShipLoader {
    type Asset = TbRelationShip;

    type Settings = ();

    type Error = TableLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::info!("TbRelationShipLoader loading start");
        let mut bytes = Vec::new();
        use bevy::asset::AsyncReadExt;
        reader.read_to_end(&mut bytes).await?;
        let t = serde_json::from_slice::<serde_json::Value>(&bytes)?;
        let tb = TbRelationShip::new(&t).unwrap();
        bevy::log::info!("TbRelationShipLoader loading over");
        Ok(tb)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}



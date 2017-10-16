extern crate config;

use std::collections::HashMap;
use config::Config;
use std::sync::RwLock;
use utils::error;
use std::path::Path;


pub static CONFIG_POOL_NAME: &'static str = "pool_name";
pub static CONFIG_POOL_CONFIG_NAME: &'static str = "config_name";
pub static CONFIG_WALLET_NAME: &'static str = "wallet_name";
pub static CONFIG_WALLET_TYPE: &'static str = "wallet_type";
pub static CONFIG_AGENT_ENDPOINT: &'static str = "agent_endpoint";

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
}

pub fn set_defaults() -> u32 {
    // if this fails the program should exit
    let mut settings = SETTINGS.write().unwrap();

    settings.set_default(CONFIG_POOL_NAME,"pool1");
    settings.set_default(CONFIG_POOL_CONFIG_NAME,"config1");
    settings.set_default(CONFIG_WALLET_NAME,"wallet1");
    settings.set_default(CONFIG_WALLET_TYPE,"default");
    settings.set_default(CONFIG_AGENT_ENDPOINT,"http://127.0.0.1:8080");

    error::SUCCESS.code_num
}

fn is_valid(value: &str) -> bool {
    for c in value.chars() {
        if !c.is_alphanumeric() && c != '_' { return false;}
    }
    true
}

fn validate_config() -> Result<u32, String> {
    let mut error = String::new();

    //if this fails the program should exit
    let config: HashMap<String, String> = SETTINGS.read().unwrap().deserialize::<HashMap<String, String>>().unwrap();

    for setting in config.iter() {
        let mut valid = true;
        if setting.0 == CONFIG_POOL_NAME && !is_valid(setting.1) {
            valid = false;
        } else if setting.0 == CONFIG_POOL_CONFIG_NAME && !is_valid(setting.1) {
            valid = false;
        } else if setting.0 == CONFIG_WALLET_NAME && !is_valid(setting.1) {
            valid = false;
        } else if setting.0 == CONFIG_AGENT_ENDPOINT {
            //TODO: validate settings for URL
        } else {
            //TODO: determine whether we should ignore invalid parameters
            //error.push_str(setting.0);
            //error.push_str("is invalid\n");
        }

        if !valid {
            error.push_str(setting.0);
            error.push_str(" has invalid setting: ");
            error.push_str(setting.1);
        }
    };

    if !error.is_empty() {
        Err(error.to_owned())
    } else {
        Ok(error::SUCCESS.code_num)
    }
}

pub fn process_config_file(path: &str) -> Result<u32, String> {

    if !Path::new(path).is_file() {
        Err("could not find configuration file".to_owned())
    } else {
        // if this fails the program should exit
        SETTINGS.write().unwrap().merge(config::File::with_name(path)).unwrap();

        match validate_config() {
            Err(x) => Err(x),
            Ok(_) => Ok(error::SUCCESS.code_num),
        }
    }
}

pub fn get_config_value(key: &str) -> Result<String, u32> {
    // if this fails the program should exit
    let config = SETTINGS.read().unwrap();

    match config.get_str(key) {
        Err(_) => Err(error::INVALID_CONFIGURATION.code_num),
        Ok(value) => Ok(value),
    }
}


#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::io::prelude::*;
    use std::fs;
    use super::*;

    #[test]
    fn test_invalid_config_value() {
        match get_config_value("garbage") {
            Err(x) => assert_eq!(x, error::INVALID_CONFIGURATION.code_num),
            Ok(v) => assert_eq!(v,"totalgarbage"), //if test gets here it will fail
        };
    }

    #[test]
    fn test_bad_path() {
        //test bad path
        let tmp_string = "garbage.txt";
        match process_config_file(&tmp_string) {
            Err(x) => assert_eq!(x,"could not find configuration file"),
            Ok(x) => assert_eq!(x,error::UNKNOWN_ERROR.code_num),  //if test gets here it will fail
        }
    }

    #[test]
    fn test_process_config_file() {
        let a = "a";
        let b = "b";

        let config_path = "/tmp/test_settings.json";
        let path = Path::new(config_path);

        let mut file = match fs::File::create(&path) {
            Err(why) => panic!("couldn't create sample config file: {}", why.description()),
            Ok(file) => file,
        };

        //throw in some invalid content to test the validation code
        let content = "{ \"a\" : \"a\", \"b\":\"b\", \"pool_name\":\"*98*\" }";

        match file.write_all(content.as_bytes()) {
            Err(why) => panic!("couldn't write to sample config file: {}", why.description()),
            Ok(_) => println!("sample config ready"),
        }

        match process_config_file(&config_path) {
            Err(_) => println!("expected invalid setting"),
            Ok(v) => assert_eq!(v, error::UNKNOWN_ERROR.code_num), //fail if we get here
        }

        match get_config_value(&a) {
            Err(x) => assert_eq!(x, error::SUCCESS.code_num), //fail if we get here
            Ok(v) => assert_eq!(v,a),
        };

        match get_config_value(&b) {
            Err(x) => assert_eq!(x, error::SUCCESS.code_num), //fail if we get here
            Ok(v) => assert_eq!(v,b),
        };

        // Leave file around or other concurrent tests will fail
        //fs::remove_file(config_path).unwrap();
    }
}
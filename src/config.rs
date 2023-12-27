use std::path::PathBuf;

use config::Config;


#[derive(Debug)]
pub struct LoadConfig { 
    path: PathBuf, 
    config_data: Config
}

impl Default for LoadConfig {
    fn default() -> Self { 
    
        let path_name = "./settings.toml";
        let path = PathBuf::from(path_name);

        let config = Config::builder()
            .add_source(config::File::with_name(path_name))
            .build()
            .expect("builder unable to parse the config file");

        Self { 
            path: path,
            config_data: config
        }
    }
}

impl LoadConfig { 
    fn new(source: impl AsRef<str>) -> Self {
        
        let path_name = source.as_ref();
        let path = PathBuf::from(path_name);

        let config = Config::builder()
            .add_source(config::File::with_name(path_name))
            .build()
            .expect("builder unable to parse the config file");

        Self { 
            path: path, 
            config_data: config
        }
    }
}
#[derive(PartialEq)]
pub enum Lang {
    EN,
    ES,
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Key {
    select_ai,
    select_ep,
    setup,
    deploy,
    deployed_api,
    select_your_data,
    folder,
    image,
    video_file,
    camera_feed,
}

pub fn translate(key: Key, lang: &Lang) -> &'static str {
    match key {
        Key::select_ai => match lang {
            Lang::EN => "Select an AI",
            Lang::ES => "Selecciona una IA",
        },
        Key::select_ep => match lang {
            Lang::EN => "Select a processor",
            Lang::ES => "Selecciona un procesador",
        }
        Key::setup => match lang {
            Lang::EN => "Setup",
            Lang::ES => "Configuración",
        }
        Key::deploy => match lang {
            Lang::EN => "Deploy",
            Lang::ES => "Desplegar",
        }
        Key::deployed_api => match lang {
            Lang::EN => "Deployed API",
            Lang::ES => "API desplegada",
        }
        Key::select_your_data => match lang {
            Lang::EN => "Select your data",
            Lang::ES => "Selecciona tus datos",
        }
        Key::folder => match lang {
            Lang::EN => "Folder",
            Lang::ES => "Carpeta",
        }
        Key::image => match lang {
            Lang::EN => "Image",
            Lang::ES => "Imagen",
        }
        Key::video_file => match lang {
            Lang::EN => "Video",
            Lang::ES => "vIDEO",
        }
        Key::camera_feed => match lang {
            Lang::EN => "Feed",
            Lang::ES => "Cámara",
        }
    }
}


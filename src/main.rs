mod core;
use core::config::Config;

use directories::{BaseDirs, ProjectDirs};
use slint::{ModelRc, SharedString, VecModel, Weak};
//use std::{path::PathBuf, rc::Rc, thread};
use std::{path::PathBuf, rc::Rc, sync::{Arc, Mutex}, thread};

slint::include_modules!(); 

fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(base) = BaseDirs::new() {
            return base.home_dir().join(rest);
        }
    }
    PathBuf::from(p)
}

fn config_base_dir() -> PathBuf {
    if let Some(proj) = ProjectDirs::from("dev", "Netherfall", "netherfall") {
        return proj.config_dir().to_path_buf();
    }
    expand_tilde("~/.netherfall")
}

fn main() -> anyhow::Result<()> {
    let ui = AppWindow::new()?;
    let weak: Weak<AppWindow> = ui.as_weak();

    let logs_store: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    ui.set_logs(Rc::new(VecModel::from(Vec::<SharedString>::new())).into());

    let logger: Arc<dyn Fn(String) + Send + Sync> = {
        let logs_store = logs_store.clone();
        let weak_for_log = weak.clone();
        Arc::new(move |text: String| {
            let snapshot: Vec<String> = {
                let mut g = logs_store.lock().unwrap();
                g.push(text);
                g.clone()
            };

            let weak2 = weak_for_log.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = weak2.upgrade() {
                    let vec_shared: Vec<SharedString> =
                        snapshot.into_iter().map(SharedString::from).collect();
                    ui.set_logs(Rc::new(VecModel::from(vec_shared)).into());
                }
            });
        })
    };

    let base = config_base_dir();
    let cfg = Config::load(base.clone()).unwrap_or_default();
    ui.set_settings_username(SharedString::from(cfg.username));
    ui.set_settings_ram_mb(cfg.ram_mb);
    ui.set_settings_compat_mode(cfg.compat_mode);
    ui.set_settings_java_path(SharedString::from(
        cfg.java_path.unwrap_or_else(|| "/usr/local/bin/java".into()),
    ));
    ui.set_settings_game_dir(SharedString::from(
        cfg.game_dir.unwrap_or_else(|| "~/.netherfall".into()),
    ));

    {
        let weak_outer = weak.clone();
        ui.on_open_game_dir(move || {
            if let Some(ui) = weak_outer.upgrade() {
                let s = ui.get_settings_game_dir().to_string();
                let path = expand_tilde(&s);
                thread::spawn(move || {
                    let _ = open::that(path);
                });
            }
        });
    }

    {
        let weak_outer = weak.clone();
        ui.on_browse_game_dir(move || {
            let init = weak_outer
                .upgrade()
                .map(|u| expand_tilde(&u.get_settings_game_dir().to_string()))
                .unwrap_or_else(|| expand_tilde("~"));

            let weak_for_thread = weak_outer.clone();
            thread::spawn(move || {
                if let Some(p) = rfd::FileDialog::new().set_directory(&init).pick_folder() {
                    let s = p.display().to_string();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak_for_thread.upgrade() {
                            ui.set_settings_game_dir(SharedString::from(s));
                        }
                    });
                }
            });
        });
    }

    {
        let weak_outer = weak.clone();
        ui.on_browse_java_path(move || {
            let init_dir = weak_outer
                .upgrade()
                .map(|u| {
                    let p = expand_tilde(&u.get_settings_java_path().to_string());
                    p.parent().unwrap_or(&p).to_path_buf()
                })
                .unwrap_or_else(|| expand_tilde("/usr/local/bin"));

            let weak_for_thread = weak_outer.clone();
            thread::spawn(move || {
                if let Some(p) = rfd::FileDialog::new().set_directory(&init_dir).pick_file() {
                    let s = p.display().to_string();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak_for_thread.upgrade() {
                            ui.set_settings_java_path(SharedString::from(s));
                        }
                    });
                }
            });
        });
    }

    {
        let weak_outer = weak.clone();
        let log = logger.clone();
        ui.on_save_settings(move || {
            if let Some(ui) = weak_outer.upgrade() {
                let mut cfg = Config::default();
                cfg.username   = ui.get_settings_username().to_string();
                cfg.ram_mb     = ui.get_settings_ram_mb();
                cfg.compat_mode = ui.get_settings_compat_mode();
                cfg.java_path  = Some(ui.get_settings_java_path().to_string());
                cfg.game_dir   = Some(ui.get_settings_game_dir().to_string());

                let base_dir = config_base_dir();
                let log2 = log.clone();                      // <—
                thread::spawn(move || {
                    let _ = cfg.save(base_dir.clone());
                    (log2)(format!("Сохранено: {}/config.toml", base_dir.display()));
                });
            }
        });
    }

    {
        let weak_outer = weak.clone();
        let log = logger.clone();
        ui.on_play(move || {
            let weak_for_thread = weak_outer.clone();
            let log2 = log.clone();                          
            thread::spawn(move || {
                (log2)("Запуск: подготовка...".into());

                let (user, java_path, game_dir, ram) = weak_for_thread
                    .upgrade()
                    .map(|u| (
                        u.get_settings_username().to_string(),
                        u.get_settings_java_path().to_string(),
                        u.get_settings_game_dir().to_string(),
                        u.get_settings_ram_mb(),
                    ))
                    .unwrap_or_else(|| ("Player".into(), "/usr/local/bin/java".into(), "~/.netherfall".into(), 4096));

                if !expand_tilde(&java_path).exists() {
                    (log2)(format!("Ошибка: Java не найден: {}", java_path));
                    (log2)("Открой «Настройки → Путь к Java» и выбери корректный файл.".into());
                    return;
                }

                (log2)(format!("Пользователь: {}", user));
                (log2)(format!("Java: {}", java_path));
                (log2)(format!("Папка игры: {}", game_dir));
                (log2)(format!("RAM: {} MB", ram));
                (log2)("OK: (демо) предпусковые проверки завершены".into());
            });
        });
    }

    {
        let weak_outer = weak.clone();
        let log = logger.clone();
        ui.on_repair(move || {
            let weak_for_thread = weak_outer.clone();
            let log2 = log.clone();                           
            thread::spawn(move || {
                (log2)("Проверка файлов игры...".into());
                let game_dir = weak_for_thread
                    .upgrade()
                    .map(|u| u.get_settings_game_dir().to_string())
                    .unwrap_or_else(|| "~/.netherfall".into());
                (log2)(format!("Каталог: {}", game_dir));
                (log2)("OK: (демо) проверка завершена".into());
            });
        });
    }

    ui.run()?;
    Ok(())
}

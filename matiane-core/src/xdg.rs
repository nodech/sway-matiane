use std::env;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct Xdg {
    pub app_name: Option<PathBuf>,
}

impl Xdg {
    pub fn new(app_name: PathBuf) -> Self {
        Xdg {
            app_name: Some(app_name),
        }
    }
}

impl Xdg {
    pub fn config_dir(&self) -> PathBuf {
        config_dir(self.app_name.as_ref())
    }

    pub fn data_dir(&self) -> PathBuf {
        data_dir(self.app_name.as_ref())
    }

    pub fn cache_dir(&self) -> PathBuf {
        cache_dir(self.app_name.as_ref())
    }

    pub fn state_dir(&self) -> PathBuf {
        state_dir(self.app_name.as_ref())
    }

    pub fn runtime_dir(&self) -> PathBuf {
        runtime_dir(self.app_name.as_ref())
    }
}

fn env_var_or_default_fn(
    env_var: &str,
    default_fn: impl FnOnce() -> PathBuf,
) -> PathBuf {
    env::var_os(env_var)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(default_fn)
}

fn env_var_or_default_home(
    env_var: &str,
    default: impl AsRef<Path>,
    app_name: Option<impl AsRef<Path>>,
) -> PathBuf {
    let mut dir = env_var_or_default_fn(env_var, || {
        env::home_dir().unwrap().join(default)
    });

    if let Some(name) = app_name {
        dir.push(name);
    }

    dir
}

pub fn config_dir(app_name: Option<impl AsRef<Path>>) -> PathBuf {
    env_var_or_default_home("XDG_CONFIG_HOME", ".config", app_name)
}

pub fn data_dir(app_name: Option<impl AsRef<Path>>) -> PathBuf {
    env_var_or_default_home("XDG_DATA_HOME", ".local/share", app_name)
}

pub fn cache_dir(app_name: Option<impl AsRef<Path>>) -> PathBuf {
    env_var_or_default_home("XDG_CACHE_HOME", ".cache", app_name)
}

pub fn state_dir(app_name: Option<impl AsRef<Path>>) -> PathBuf {
    env_var_or_default_home("XDG_STATE_HOME", ".local/state", app_name)
}

pub fn runtime_dir(app_name: Option<impl AsRef<Path>>) -> PathBuf {
    let mut dir = env_var_or_default_fn("XDG_RUNTIME_DIR", env::temp_dir);

    if let Some(name) = app_name {
        dir.push(name);
    }

    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    macro_rules! gen_env_test {
        (
            $test_name:ident,
            $fn_name: ident,
            $env_var:literal,
            $default:literal
        ) => {
            #[test]
            fn $test_name() -> Result<()> {
                unsafe {
                    env::set_var("HOME", "/home/test");
                }

                unsafe {
                    env::set_var($env_var, "/home/test/custom");
                }

                let result_not_set = $fn_name(Some("app"));
                assert_eq!(
                    result_not_set,
                    PathBuf::from("/home/test/custom/app")
                );

                // test not set
                unsafe {
                    env::remove_var($env_var);
                }

                let result_not_set = $fn_name(Some("app"));
                assert_eq!(
                    result_not_set,
                    PathBuf::from(format!("/home/test/{}/app", $default))
                );

                Ok(())
            }
        };
    }

    #[test]
    fn env_var_or_default_test() -> Result<()> {
        unsafe {
            env::set_var("HOME", "/home/test");
            env::set_var("EXISTS", "/test");
        }

        {
            let result = env_var_or_default_home(
                "HOPEFULLY_THIS_DOES_NOT_EXIST",
                "child",
                None::<&str>,
            );
            assert_eq!(result, PathBuf::from("/home/test/child"));
        }

        {
            let exists =
                env_var_or_default_home("EXISTS", "child2", None::<&str>);
            assert_eq!(exists, PathBuf::from("/test"));
        }

        Ok(())
    }

    gen_env_test!(config_dir_test, config_dir, "XDG_CONFIG_HOME", ".config");
    gen_env_test!(data_dir_test, data_dir, "XDG_DATA_HOME", ".local/share");
    gen_env_test!(cache_dir_test, cache_dir, "XDG_CACHE_HOME", ".cache");
    gen_env_test!(state_dir_test, state_dir, "XDG_STATE_HOME", ".local/state");
}

use crate::config::Configuration;
use crate::errors::Errcode;

#[allow(unused_imports)]
use crate::{loader::Loader, render::Render};

pub fn setup_files(config: &Configuration) -> Result<(), Errcode> {
    std::fs::create_dir_all(&config.assets_dir)?;

    std::fs::copy(
        &config.site_config.favicon,
        config.assets_dir.join("favicon"),
    )?;

    fs_extra::copy_items(
        &config.add_assets,
        &config.assets_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn reload(ldr: &Loader, rdr: &Render, config: &Configuration) -> Result<(), Errcode> {
    let tstart = std::time::Instant::now();
    rdr.engine.write().full_reload()?;
    Render::setup_css(config)?;
    Render::setup_scripts(config)?;
    log::debug!("Engine reload in {:?}", tstart.elapsed());
    setup_files(config)?;
    log::debug!("Setup files in {:?}", tstart.elapsed());
    ldr.reload()?;
    log::debug!("Storage reload in {:?}", tstart.elapsed());
    Ok(())
}

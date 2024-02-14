use crate::config::Configuration;
use crate::errors::Errcode;

#[allow(unused_imports)]
use crate::{loader::Loader, render::Render};

pub async fn setup_files(config: &Configuration) -> Result<(), Errcode> {
    tokio::fs::create_dir_all(&config.assets_dir).await?;

    tokio::fs::copy(
        &config.site_config.favicon,
        config.assets_dir.join("favicon.ico"),
    )
    .await?;

    fs_extra::copy_items(
        &config.add_assets,
        &config.assets_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;
    Ok(())
}

#[allow(dead_code)]
pub async fn reload(ldr: &Loader, rdr: &Render, config: &Configuration) -> Result<(), Errcode> {
    let tstart = std::time::Instant::now();
    rdr.engine.write().full_reload()?;
    Render::setup_css(config).await?;
    Render::setup_scripts(config).await?;
    log::debug!("Engine reload in {:?}", tstart.elapsed());
    setup_files(config).await?;
    log::debug!("Setup files in {:?}", tstart.elapsed());
    ldr.reload().await?;
    log::debug!("Storage reload in {:?}", tstart.elapsed());

    #[cfg(feature = "add-endpoint")]
    crate::extensions::addendpoint::register_templates(config, rdr)?;

    Ok(())
}

use std::io::Write;

use crate::store::*;

pub fn download_all_comics(redownload: bool) -> eyre::Result<()> {
    println!("Fetching latest comic ...");
    let latest_comic = Comic::get_from_url("https://xkcd.com/info.0.json")?;

    let count = latest_comic.num;

    println!("There are {count} comics (excluding 404)");

    let mut store = if redownload {
        Store::default()
    } else {
        Store::load()
    };

    for i in 1..=count {
        if store.has_comic(i)
            && store
                .fetch_comic(i)
                .is_ok_and(|comic| comic.has_image_downloaded())
        {
            continue;
        }

        println!("Fetching comic #{i} ...");
        match store.fetch_comic(i) {
            Ok(comic) => {
                if redownload || !comic.has_image_downloaded() {
                    print!("Downloading image #{i} ...");
                    let _ = std::io::stdout().flush();
                    match comic.download_image() {
                        Ok(()) => println!(" done!"),
                        Err(e) => println!(" error: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("error fetching comic #{i}: {e}"),
        }

        // if i % 10 == 0 {
        //     println!("Saving database backup ...");
        //     store.save();
        // }
    }

    println!("Done fetching all comics!");

    store.save();

    Ok(())
}

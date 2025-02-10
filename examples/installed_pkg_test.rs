use installed_pkg::list as app_list;
use vat::noduro;
use std::env;
fn main(){
    // let installed = app_list();
    // if let Ok(installed) = installed{
    //     for app in installed.apps_sys.iter(){
    //         println!("app: {:?}", app.root);
    //     }
    // }

    // if let Ok(program_files) = env::var("ProgramFiles") {
    //     println!("Program Files: {}", program_files);
    // } else {
    //     println!("Could not find Program Files directory.");
    // }

    // let apps = installed::list();
    // if let Ok(apps) = apps{
    //     let re = Regex::new(r#"UninstallString: "\\\\([^"]+)""#).unwrap();
    //     for app in apps{
    //         // println!("app: {:?}, version: {:?}", app.name(), app.version());
    //         // println!("app: {:?}", app.publisher());
    //         // println!("app: {:?}", app.dump());
    //         // break;
    //         if let Some(caps) = re.captures(&app.dump()) {
    //             println!("Path: {}", &caps[1]);
    //         }
    //         break;
    //     }
    // }


    let apps = noduro::find_installed_apps();
    let filtered_apps = noduro::filter_apps(apps);
    // for app in filtered_apps{
    //     println!("app: {:?}", app);
    // }

    let manual_apps = noduro::manual_checking();
    let mut final_apps = Vec::new();
    for app in filtered_apps{
        final_apps.push(app);
    }
    for app in manual_apps{
        final_apps.push(app);
    }


    let dcc_package = noduro::dcc_package_from_apps(final_apps);
    dbg!(&dcc_package);
}
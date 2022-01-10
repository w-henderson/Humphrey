use humphrey::handlers::serve_file;
use humphrey::{App, SubApp};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let localhost_subapp = SubApp::new()
        .with_route("/different_response", serve_file("./static/localhost.html"))
        .with_route("/localhost_only", serve_file("./static/localhost.html"));

    let localip_subapp = SubApp::new()
        .with_route("/different_response", serve_file("./static/localip.html"))
        .with_route("/localip_only", serve_file("./static/localip.html"));

    let app: App<()> = App::new()
        .with_route("/", serve_file("./static/index.html"))
        .with_host("localhost", localhost_subapp)
        .with_host("127.0.0.1", localip_subapp);

    app.run("0.0.0.0:80")?;

    Ok(())
}

mod api;
mod dashboard;

use topcoat::{
    Result,
    router::layout,
    tailwind,
    view::view,
};

#[layout("/")]
async fn root(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Parallax"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::dev::script()
            </head>
            <body class="bg-zinc-950 text-zinc-100 min-h-screen antialiased">
                (slot?)
            </body>
        </html>
    }
}

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod pages;
use components::{Header, Footer};

use pages::HomePage;
use pages::ComposePage;
use pages::AccountsPage;
use pages::ProfilePage;
use pages::DocsPage;
use pages::LoginPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Title text="W9 Mail"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Stylesheet id="voxel" href="/pkg/w9-mail-client.css"/>
        <Router>
            <div class="app-container">
                <Header/>
                <main class="main-content">
                    <Routes>
                        <Route path="home" view=HomePage/>
                        <Route path="compose" view=ComposePage/>
                        <Route path="accounts" view=AccountsPage/>
                        <Route path="profile" view=ProfilePage/>
                        <Route path="docs" view=DocsPage/>
                        <Route path="login" view=LoginPage/>
                    </Routes>
                </main>
                <Footer/>
            </div>
        </Router>
    }
}

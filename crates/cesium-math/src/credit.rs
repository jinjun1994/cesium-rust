/// A credit contains data pertaining to how to display attributions/credits for certain content on the screen.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Credit {
    html: String,
    show_on_screen: bool,
}

impl Credit {
    pub fn new(html: impl Into<String>, show_on_screen: Option<bool>) -> Self {
        Self {
            html: html.into(),
            show_on_screen: show_on_screen.unwrap_or(false),
        }
    }

    pub fn html(&self) -> &str {
        &self.html
    }

    pub fn show_on_screen(&self) -> bool {
        self.show_on_screen
    }
}

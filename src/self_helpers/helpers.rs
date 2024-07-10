use crate::TemplateApp;

impl TemplateApp {

    pub fn username(&mut self) -> String {
        if let Some(you) = &self.you {
            you.username.clone()
        } else {
            String::from("you")
        }
    }

    pub fn /*baba_*/is_you(&self, id: &String) -> bool {
        if let Some(you_id) = &self.user_id {
            you_id.eq(id)
        } else {
            false
        }
    }
}
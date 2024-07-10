use crate::{FrontendPage, TemplateApp};

impl TemplateApp {

    pub fn current_page(&self) -> &FrontendPage {
        self.page_stack.get(self.current_page).or(Some(&FrontendPage::UnknownPage)).unwrap()
    }

    pub fn set_page(&mut self, page: FrontendPage) {
        if self.current_page != self.page_stack.len() - 1 { // if we're not on the latest page, remove the "future" pages
            self.page_stack.truncate(self.current_page+1);
        }
        self.current_page = self.page_stack.len(); // length is 1-indexed so by setting before we add the new one, the index is right
        self.page_stack.push(page);
    }

    pub fn page_back(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    pub fn page_forward(&mut self) {
        if self.current_page < self.page_stack.len() - 1 {
            self.current_page += 1;
        }
    }
}
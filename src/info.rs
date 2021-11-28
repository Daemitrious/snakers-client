#[derive(Clone)]
struct Pair<T: AsRef<str> + Clone> {
    name: T,
    data: T,
}

impl<T: AsRef<str> + Clone> Pair<T> {
    fn to_string(&self) -> String {
        format!("{}: {}", self.name.as_ref(), self.data.as_ref())
    }
}

#[derive(Clone)]
pub struct Section<T: AsRef<str> + Clone> {
    data: Vec<Pair<T>>,
    parent: Category<T>,
}

impl<T: AsRef<str> + Clone> Section<T> {
    pub fn add_pair(mut self, name: T, data: T) -> Self {
        self.data.push(Pair { name, data });
        self
    }

    pub fn finalize(mut self) -> Category<T> {
        self.parent.data.push(self.clone());
        self.parent
    }

    fn to_string(&self) -> String {
        let mut data = "".to_owned();
        for pair in self.data.iter() {
            data.push_str("\n\t\t");
            data.push_str(&pair.to_string())
        }
        data
    }
}

#[derive(Clone)]
pub struct Category<T: AsRef<str> + Clone> {
    name: T,
    data: Vec<Section<T>>,
    parent: Info<T>,
}

impl<T: AsRef<str> + Clone> Category<T> {
    pub fn new_section(self) -> Section<T> {
        Section {
            data: Vec::new(),
            parent: self,
        }
    }

    pub fn finalize(mut self) -> Info<T> {
        self.parent.data.push(self.clone());
        self.parent
    }

    fn to_string(&self) -> String {
        let mut data = "".to_owned();

        for i in 0..self.data.len() {
            data.push_str("\t");
            data.push_str(&self.data[i].to_string());
        }

        format!("{}:{}", self.name.as_ref(), data)
    }
}

#[derive(Clone)]
pub struct Info<T: AsRef<str> + Clone> {
    name: T,
    data: Vec<Category<T>>,
    note: Option<T>,
}

impl<T: AsRef<str> + Clone> Info<T> {
    pub fn new(name: T) -> Self {
        Self {
            name,
            data: Vec::new(),
            note: None,
        }
    }

    pub fn new_category(self, name: T) -> Category<T> {
        Category {
            name,
            data: Vec::new(),
            parent: self,
        }
    }

    pub fn finalize(self, note: Option<T>) -> String {
        let mut data = "".to_owned();
        for category in self.data.iter() {
            data.push_str("\n\t");
            data.push_str(&category.to_string());
        }

        if let Some(content) = note {
            data.push_str("\n\n");
            data.push_str(content.as_ref())
        }

        format!("{}:{}", self.name.as_ref(), data)
    }
}

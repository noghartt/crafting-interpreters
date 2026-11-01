#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub file: FileId,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(usize);

pub struct SourceFile {
    pub path: Option<String>,
    pub chars: Vec<char>,
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, path: Option<String>, text: &str) -> FileId {
        let id = FileId(self.files.len());
        self.files.push(SourceFile {
            path,
            chars: text.chars().collect(),
        });

        id
    }

    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.0]
    }
}

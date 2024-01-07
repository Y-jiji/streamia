use std::{collections::{HashSet, HashMap, BTreeMap, BinaryHeap}, cmp::Reverse};

#[derive(Debug, Clone)]
pub struct Note {
    id: usize,
    pub fw: usize,   // forward to which node?
    pub tag: String,
    pub del: bool,   // if current note is deleted
    pub muted: bool, // if current note is muted
    pub pitch: f64,
    pub range: (u64, u64),
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.range.1 == other.range.1
    }
}

impl Eq for Note {}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.range.1.cmp(&other.range.1))
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.range.1.cmp(&other.range.1)
    }
}

pub struct Sheet {
    idmap: HashMap<usize, usize>,
    count: usize,
    inner: Vec<Note>,
    segtree_index: SegTree,
    trigger_index: BTreeMap<u64, HashSet<usize>>,
}

pub enum SheetError {
    NoSuchNote(usize),
}

type Result<T> = std::result::Result<T, SheetError>;

impl Sheet {
    pub fn new() -> Sheet {
        Self {
            count: 0, inner: vec![],
            idmap: HashMap::new(), 
            segtree_index: SegTree::new(0),
            trigger_index: BTreeMap::new(),
        }
    }
    pub fn player(&self, start: u64) -> SheetPlayer<'_> {
        SheetPlayer::new(&self, start)
    }
    pub fn put(&mut self, mut note: Note) -> Result<()> {
        assert!(note.del != false);
        note.id = self.count;
        self.idmap.insert(self.count, self.inner.len());
        self.count += 1;
        self.segtree_index.put(self.inner.len(), note.range);
        self.trigger_index.entry(note.range.0)
            .and_modify(|s| {s.insert(self.inner.len());})
            .or_insert(HashSet::from_iter([self.inner.len()]));
        self.inner.push(note);
        Ok(())
    }
    pub fn rm(&mut self, i: usize) -> Result<()> {
        let i = self.idmap.remove(&i).ok_or(SheetError::NoSuchNote(i))?;
        self.inner[i].del = true;
        self.segtree_index.rm(i, self.inner[i].range);
        self.trigger_index.entry(self.inner[i].range.0)
            .and_modify(|s| {s.remove(&i);});
        Ok(())
    }
    pub fn mute(&mut self, i: usize) -> Result<()> {
        let &i = self.idmap.get(&i).ok_or(SheetError::NoSuchNote(i))?;
        self.inner[i].muted = true;
        Ok(())
    }
    pub fn unmute(&mut self, i: usize) -> Result<()> {
        let &i = self.idmap.get(&i).ok_or(SheetError::NoSuchNote(i))?;
        self.inner[i].muted = false;
        Ok(())
    }
}

pub struct SegTree {
    tie: u64, set: HashSet<usize>,
    inner: Box<[Option<SegTree>; 1 << 4]>,
    count: usize,
}

impl SegTree {
    pub fn new(tie: u64) -> SegTree {
        Self {
            tie, set: HashSet::new(), count: 0,
            inner: Box::new([0; 1<<4].map(|_| None))
        }
    }
    pub fn get(&self, t: u64) -> Vec<usize> {
        let mut set = self.inner[((t >> self.tie) & 0xf) as usize].as_ref()
            .map(|tree| tree.get(t)).unwrap_or(Vec::new());
        set.extend(self.set.iter().copied());
        return set;
    }
    pub fn put(&mut self, i: usize, t: (u64, u64)) {
        let range = 
            (t.0 >> self.tie & 0xf) ..= 
            (t.1 >> self.tie & 0xf);
        self.count += 1;
        if range == (0x0 ..= 0xf) || u64::MAX >> self.tie == 0 {
            self.set.insert(i); return
        }
        for j in range.map(|j| j as usize) {
            if self.inner[j].is_none() {
                self.inner[j] = Some(SegTree::new(self.tie + 4));
            }
            self.inner[j].as_mut().unwrap().put(i, t);
        }
    }
    pub fn rm(&mut self, i: usize, t: (u64, u64)) {
        let range = 
            (t.0 >> self.tie & 0xf) ..= 
            (t.1 >> self.tie & 0xf);
        self.count -= 1;
        if range == (0x0 ..= 0xf) || u64::MAX >> self.tie == 0 {
            self.set.remove(&i); return
        }
        for j in range.map(|j| j as usize) {
            let count = self.inner[j].as_mut()
                .map(|tree| {tree.rm(i, t); tree.count});
            if count == Some(0) { self.inner[j] = None }
        }
    }
}

pub struct SheetPlayer<'a> {
    sheet: &'a Sheet, 
    cursor: u64,
    active: BinaryHeap<Reverse<&'a Note>>,
}

impl<'a> SheetPlayer<'a> {
    fn new(sheet: &'a Sheet, start: u64) -> Self {
        let active = sheet.segtree_index.get(start)
            .into_iter().map(|i| Reverse(&sheet.inner[i])).collect();
        SheetPlayer { sheet, cursor: start, active }
    }
    fn get_active(&self) -> impl Iterator<Item=&Note> {
        self.active.iter().map(|Reverse(x)| *x)
    }
    fn get_cursor(&self) -> u64 {
        self.cursor
    }
    fn tick_by(&mut self, delta: u64) {
        while matches!(self.active.peek(), Some(Reverse(note)) if note.range.1 >= self.cursor + delta) {
            self.active.pop();
        }
        self.active.extend(self.sheet
            .trigger_index
            .range(self.cursor..self.cursor+delta)
            .flat_map(|(_, x)| x.iter())
            .map(|&i| Reverse(&self.sheet.inner[i]))
        );
        self.cursor += delta;
    }
}
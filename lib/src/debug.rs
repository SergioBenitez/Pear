use std::cell::RefCell;
use std::collections::HashMap;

type Index = usize;

struct Tree<T> {
    // All of the nodes in the tree live in this vector.
    nodes: Vec<T>,
    // Maps from an index (`parent`) index `nodes` to a set of indexes in
    // `nodes` corresponding to the children of `key`.
    children: HashMap<Index, Vec<Index>>,
    // This "tree" keeps track of which parent children are currently being
    // pushed to. A `push` adds to this stack while a `pop` removes from this
    // stack. If the stack is empty, the root is being pushed to.
    stack: Vec<Index>
}

impl<T> Tree<T> {
    fn new() -> Tree<T> {
        Tree {
            nodes: vec![],
            children: HashMap::new(),
            stack: Vec::with_capacity(8)
        }
    }

    fn push(&mut self, node: T) -> Index {
        // Add the node to the tree and get its index.
        self.nodes.push(node);
        let index = self.nodes.len() - 1;

        // If the stack indicates we have a parent, add to its children.
        if !self.stack.is_empty() {
            let parent = self.stack[self.stack.len() - 1];
            self.children.entry(parent).or_insert(vec![]).push(index);
        }

        // Make this the new parent.
        self.stack.push(index);
        index
    }

    fn pop_level(&mut self) -> Option<Index> {
        self.stack.pop()
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn get(&self, index: Index) -> &T {
        &self.nodes[index]
    }

    fn get_mut(&mut self, index: Index) -> &mut T {
        &mut self.nodes[index]
    }

    fn get_children(&self, index: Index) -> &[Index] {
        match self.children.get(&index) {
            Some(children) => &children[..],
            None => &[]
        }
    }
}

struct Info {
    name: &'static str,
    success: Option<bool>,
    start_context: Option<String>,
    end_context: Option<String>,
}

impl Info {
    fn new(name: &'static str, start_context: Option<String>) -> Info {
        Info { name, start_context, success: None, end_context: None }
    }
}

thread_local! {
    #[doc(hidden)]
    static PARSE_TREE: RefCell<Tree<Info>> = RefCell::new(Tree::new());
}

fn debug_print(sibling_map: &mut Vec<bool>, node: Index) {
    let parent_count = sibling_map.len();
    for (i, &has_siblings) in sibling_map.iter().enumerate() {
        if i < parent_count - 1 {
            match has_siblings {
                true => print!(" │   "),
                false => print!("     ")
            }
        } else {
            match has_siblings {
                true => print!(" ├── "),
                false => print!(" └── ")
            }
        }
    }

    PARSE_TREE.with(|key| {
        let tree = key.borrow();
        let info = tree.get(node);
        let success = match info.success {
            Some(true) => " ✓",
            Some(false) => " ✗",
            None => ""
        };

        let ctxt = match (&info.start_context, &info.end_context) {
            (&Some(ref a), &Some(ref b)) => format!(" [{}] - [{}]", a, b),
            _ => "".into()
        };

        println!("{}{}{}", info.name, success, ctxt);
        let children = tree.get_children(node);
        let num_children = children.len();
        for (i, &child) in children.iter().enumerate() {
            let have_siblings = i != (num_children - 1);
            sibling_map.push(have_siblings);
            debug_print(sibling_map, child);
            sibling_map.pop();
        }
    });
}

#[doc(hidden)]
pub fn parser_entry(name: &'static str, ctxt: Option<String>) {
    if is_debug!() {
        PARSE_TREE.with(|key| key.borrow_mut().push(Info::new(name, ctxt)));
    }
}

#[doc(hidden)]
pub fn parser_exit(_: &'static str, success: bool, ctxt: Option<String>) {
    if is_debug!() {
        let done = PARSE_TREE.with(|key| {
            // FIXME: Record whether it was successful or not.
            let mut tree = key.borrow_mut();
            let index = tree.pop_level();
            if let Some(last_node) = index {
                let last = tree.get_mut(last_node);
                last.success = Some(success);
                last.end_context = ctxt;
            }

            index
        });

        // We've reached the end. Print the whole thing and clear the tree.
        if let Some(0) = done {
            debug_print(&mut vec![], 0);
            PARSE_TREE.with(|key| key.borrow_mut().clear());
        }
    }
}

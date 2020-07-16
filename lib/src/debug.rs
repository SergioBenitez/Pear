use std::cell::RefCell;
use std::collections::HashMap;

use crate::input::{Show, ParserInfo};
use crate::macros::is_parse_debug;

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
    parser: ParserInfo,
    context: Option<String>,
    success: Option<bool>,
}

impl Info {
    fn new(parser: ParserInfo) -> Info {
        Info { parser, context: None, success: None }
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

        #[cfg(feature = "color")]
        let color = match info.success {
            Some(true) => ::yansi::Color::Green,
            Some(false) => ::yansi::Color::Red,
            None => ::yansi::Color::Unset,
        };

        let ctxt = match info.context {
            Some(ref context) => context.to_string(),
            _ => "".to_string()
        };

        #[cfg(feature = "color")] {
            println!("{} ({})",
                     color.paint(format!("{}{}", info.parser.name, success)),
                     ctxt);
        }

        #[cfg(not(feature = "color"))]
        println!("{}{} ({})", info.name, success, ctxt);

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

// TODO: Take in &[&dyn Show] to display parser input parameters.
#[doc(hidden)]
pub fn parser_entry(parser: &ParserInfo) {
    if (parser.raw && is_parse_debug!("full")) || (!parser.raw && is_parse_debug!()) {
        PARSE_TREE.with(|key| key.borrow_mut().push(Info::new(*parser)));
    }
}

#[doc(hidden)]
pub fn parser_exit(parser: &ParserInfo, success: bool, ctxt: Option<&dyn Show>) {
    if (parser.raw && is_parse_debug!("full")) || (!parser.raw && is_parse_debug!()) {
        let done = PARSE_TREE.with(|key| {
            let mut tree = key.borrow_mut();
            let index = tree.pop_level();
            if let Some(last_node) = index {
                let last = tree.get_mut(last_node);
                last.success = Some(success);
                last.context = ctxt.map(|c| c.to_string());
            }

            index
        });

        // We've reached the end. Print the whole thing and clear the tree.
        if let Some(0) = done {
            #[cfg(feature = "color")] {
                if cfg!(windows) && !::yansi::Paint::enable_windows_ascii() {
                    ::yansi::Paint::disable();
                }
            }

            debug_print(&mut vec![], 0);
            PARSE_TREE.with(|key| key.borrow_mut().clear());
        }
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
static DEBUG_CONTEXT: AtomicBool = AtomicBool::new(true);

#[cfg(not(debug_assertions))]
static DEBUG_CONTEXT: AtomicBool = AtomicBool::new(false);

#[inline(always)]
pub fn enable_context(enable: bool) {
    DEBUG_CONTEXT.store(enable, Ordering::Release)
}

#[inline(always)]
pub fn context_enabled() -> bool {
    DEBUG_CONTEXT.load(Ordering::Acquire)
}

// FIXME: Remove the global state with a wrapping input like the one below.
// Major caveat: the blanket Token impls in `input` prevent a blanket input
// here.

// pub struct Debug<I> {
//     input: I,
//     tree: Tree<Info>,
// }

// use crate::input::{Input, ParserInfo, Token};

// impl<I: Input> Input for Debug<I> {
//     type Token = I::Token;
//     type Slice = I::Slice;
//     type Many = I::Many;

//     type Marker = I::Marker;
//     type Context = I::Context;

//     fn token(&mut self) -> Option<Self::Token> {
//         self.input.token()
//     }

//     fn slice(&mut self, n: usize) -> Option<Self::Slice> {
//         self.input.slice(n)
//     }

//     fn peek<F>(&mut self, cond: F) -> bool
//         where F: FnMut(&Self::Token) -> bool
//     {
//         self.input.peek(cond)
//     }

//     fn peek_slice<F>(&mut self, n: usize, cond: F) -> bool
//         where F: FnMut(&Self::Slice) -> bool
//     {
//         self.input.peek_slice(n, cond)
//     }

//     fn eat<F>(&mut self, cond: F) -> Option<Self::Token>
//         where F: FnMut(&Self::Token) -> bool
//     {
//             self.input.eat(cond)
//     }

//     fn eat_slice<F>(&mut self, n: usize, cond: F) -> Option<Self::Slice>
//         where F: FnMut(&Self::Slice) -> bool
//     {
//         self.input.eat_slice(n, cond)
//     }

//     fn take<F>(&mut self, cond: F) -> Self::Many
//         where F: FnMut(&Self::Token) -> bool
//     {
//         self.input.take(cond)
//     }

//     fn skip<F>(&mut self, cond: F) -> usize
//         where F: FnMut(&Self::Token) -> bool
//     {
//         self.input.skip(cond)
//     }

//     fn is_eof(&mut self) -> bool {
//         self.input.is_eof()
//     }

//     fn mark(&mut self, info: &ParserInfo) -> Option<Self::Marker> {
//         self.input.mark(info)
//     }

//     fn context(&mut self, mark: Option<&Self::Marker>) -> Option<Self::Context> {
//         self.input.context(mark)
//     }

//     fn unmark(&mut self, info: &ParserInfo, success: bool, mark: Option<Self::Marker>) {
//         self.input.unmark(info, success, mark)
//     }
// }


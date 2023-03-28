// Based on ammonia (https://github.com/rust-ammonia/ammonia)
//
// License for ammonia:
//
// Copyright (c) 2015-2022 The ammonia Developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod dom;

#[cfg(test)]
mod tests;

use aho_corasick::AhoCorasickBuilder;
use dom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use html5ever::{
    driver,
    interface::tree_builder::{NodeOrText, TreeSink},
    local_name, namespace_url, ns,
    serialize::{serialize, SerializeOpts},
    tendril::*,
    Attribute, QualName,
};
use std::{
    cell::RefCell,
    fmt::{self, Display},
    mem,
    rc::Rc,
};

#[derive(Debug, Default)]
pub struct Builder<'a> {
    search_term: Option<&'a str>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn search_term(&mut self, value: Option<&'a str>) -> &mut Self {
        self.search_term = value;
        self
    }

    fn search_child(&self, _child: &mut Handle) -> bool {
        true
    }

    pub fn search_dom(&self, mut dom: RcDom) -> Document {
        let mut stack = Vec::new();
        let mut removed = Vec::new();

        let body = {
            let children = dom.document.children.borrow();
            children[0].clone()
        };
        stack.extend(
            mem::take(&mut *body.children.borrow_mut())
                .into_iter()
                .rev(),
        );
        let mut already_matched = false;

        while let Some(mut node) = stack.pop() {
            let parent = node.parent.replace(None).expect("a node in the DOM will have a parent, except the root, which is not searched")
                .upgrade().expect("a node's parent will be pointed to by its parent (or the root pointer), and will not be dropped");
            let pass_search = self.search_child(&mut node);
            if pass_search {
                if self.search_term.is_some() {
                    if let Some(value) =
                        self.replacement_node(&mut node, &mut dom, &mut already_matched)
                    {
                        // node should be a TextNode and so have no children to check so OK to
                        // continue here
                        for new_child_node in value.iter() {
                            dom.append(&parent, NodeOrText::AppendNode(new_child_node.clone()));
                        }
                        removed.push(node);
                        continue;
                    };
                };
                dom.append(&parent.clone(), NodeOrText::AppendNode(node.clone()));
            } else {
                for sub in node.children.borrow_mut().iter_mut() {
                    sub.parent.replace(Some(Rc::downgrade(&parent)));
                }
            }
            stack.extend(
                mem::take(&mut *node.children.borrow_mut())
                    .into_iter()
                    .rev(),
            );
        }
        while let Some(node) = removed.pop() {
            removed.extend_from_slice(&mem::take(&mut *node.children.borrow_mut())[..]);
        }
        Document(dom)
    }

    pub fn search(&self, src: &str) -> Document {
        let parser = Self::make_parser();
        let dom = parser.one(src);
        self.search_dom(dom)
    }

    /*
     * Searches text content within `child` for the search term. Returns `None` if no match is
     * found and returns `Some(replacement)` if a match is found. `replacement` will have occurrences
     * of the search term wrapped in a `<mark>` tag.
     */
    fn replacement_node(
        &self,
        child: &mut Handle,
        dom: &mut RcDom,
        already_matched: &mut bool,
    ) -> Option<Vec<Rc<Node>>> {
        let mut replacement_nodes = Vec::new();
        if let NodeData::Text { ref contents, .. } = child.data {
            let search_pattern: Vec<&str> = self.search_term?.split(' ').collect();
            let ac = AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(search_pattern);
            let mut matches = vec![];
            let search_content = contents.borrow();
            for search_term_match in ac.find_iter(&search_content[..]) {
                matches.push((search_term_match.start(), search_term_match.end()));
            }
            let mut index: usize = 0;
            for (start, end) in matches.iter() {
                replacement_nodes.push(Node::new(NodeData::Text {
                    contents: RefCell::new(search_content[index..*start].into()),
                }));
                let new_mark_node_text = Node::new(NodeData::Text {
                    contents: RefCell::new(search_content[*start..*end].into()),
                });
                let new_mark_node = if *already_matched {
                    Node::new(NodeData::Element {
                        name: QualName::new(None, ns!(), local_name!("mark")),
                        attrs: RefCell::new(vec![]),
                        template_contents: RefCell::new(None),
                        mathml_annotation_xml_integration_point: false,
                    })
                } else {
                    let search_attribute = Attribute {
                        name: QualName::new(None, ns!(), local_name!("id")),
                        value: "search-match".into(),
                    };
                    *already_matched = true;
                    Node::new(NodeData::Element {
                        name: QualName::new(None, ns!(), local_name!("mark")),
                        attrs: RefCell::new(vec![search_attribute]),
                        template_contents: RefCell::new(None),
                        mathml_annotation_xml_integration_point: false,
                    })
                };
                dom.append(&new_mark_node, NodeOrText::AppendNode(new_mark_node_text));
                replacement_nodes.push(new_mark_node);
                index = *end;
            }
            replacement_nodes.push(Node::new(NodeData::Text {
                contents: RefCell::new(search_content[index..].into()),
            }));
            if replacement_nodes.is_empty() {
                return None;
            } else {
                return Some(replacement_nodes);
            }
        }
        None
    }

    pub fn make_parser() -> driver::Parser<RcDom> {
        driver::parse_fragment(
            RcDom::default(),
            driver::ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("div")),
            vec![],
        )
    }
}

pub struct Document(RcDom);

impl Document {
    fn serialize_opts() -> SerializeOpts {
        SerializeOpts::default()
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        let parser = Builder::make_parser();
        let dom = parser.one(&self.to_string()[..]);
        Document(dom)
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opts = Self::serialize_opts();
        let mut ret_val = Vec::new();
        let inner: SerializableHandle = self.0.document.children.borrow()[0].clone().into();
        serialize(&mut ret_val, &inner, opts)
            .expect("Writing to a string shouldn't fail (expect on OOM)");
        String::from_utf8(ret_val)
            .expect("html5ever only supports UTF8")
            .fmt(f)
    }
}

pub fn search_html(html: &str, search_term: &str) -> String {
    Builder::new()
        .search_term(Some(search_term))
        .search(html)
        .to_string()
}

mod dom;

#[cfg(test)]
mod tests;

use aho_corasick::AhoCorasickBuilder;
use dom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use html5ever::{
    driver,
    interface::tree_builder::{AppendNode, NodeOrText, TreeSink},
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
                        // unload existing children to temporary node
                        let temp_node = Node::new(NodeData::Element {
                            name: QualName::new(None, ns!(), local_name!("div")),
                            attrs: RefCell::new(vec![]),
                            template_contents: RefCell::new(None),
                            mathml_annotation_xml_integration_point: false,
                        });
                        dom.reparent_children(&node, &temp_node);

                        // add replacement children to node
                        dom.reparent_children(&value, &node);
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
    ) -> Option<Rc<Node>> {
        if let NodeData::Element { ref name, .. } = child.data {
            let child_local_name = &*name.local;
            if child_local_name != "mark" {
                let replacement_node = Node::new(NodeData::Element {
                    name: QualName::new(None, ns!(), name.local.clone()),
                    attrs: RefCell::new(vec![]),
                    template_contents: RefCell::new(None),
                    mathml_annotation_xml_integration_point: false,
                });
                let search_pattern: Vec<&str> = self.search_term?.split(' ').collect();
                let ac = AhoCorasickBuilder::new()
                    .ascii_case_insensitive(true)
                    .build(search_pattern);
                for grandchild in child.children.borrow().iter() {
                    if let NodeData::Text { ref contents } = grandchild.data {
                        let mut matches = vec![];
                        let search_content = contents.borrow();
                        for search_term_match in ac.find_iter(&search_content[..]) {
                            matches.push((search_term_match.start(), search_term_match.end()));
                        }
                        let mut index: usize = 0;
                        for (start, end) in matches.iter() {
                            dom.append(
                                &replacement_node,
                                AppendNode(Node::new(NodeData::Text {
                                    contents: RefCell::new(search_content[index..*start].into()),
                                })),
                            );
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
                            dom.append(&replacement_node, NodeOrText::AppendNode(new_mark_node));
                            index = *end;
                        }
                        dom.append(
                            &replacement_node,
                            AppendNode(Node::new(NodeData::Text {
                                contents: RefCell::new(search_content[index..].into()),
                            })),
                        );
                    }
                }
                if replacement_node.children.borrow().is_empty() {
                    return None;
                } else {
                    return Some(replacement_node);
                }
            }
        }
        None
    }

    //     fn replacement_node(&self, child: &mut Handle, dom: &mut RcDom) -> Option<Rc<Node>> {
    //         if let NodeData::Element { ref name, .. } = child.data {
    //             if &*name.local == "p" {
    //                 let new_paragraph_node = Node::new(NodeData::Element {
    //                     name: QualName::new(None, ns!(), local_name!("p")),
    //                     attrs: RefCell::new(vec![]),
    //                     template_contents: RefCell::new(None),
    //                     mathml_annotation_xml_integration_point: false,
    //                 });
    //                 for grandchild in child.children.borrow().iter() {
    //                     if let NodeData::Text { ref contents } = grandchild.data {
    //                         let search_pattern = self.search_term?;
    //                         let ac = AhoCorasickBuilder::new()
    //                             .ascii_case_insensitive(true)
    //                             .build(vec![search_pattern]);
    //                         let mut matches = vec![];
    //                         let search_content = contents.borrow();
    //                         for search_term_match in ac.find_iter(&search_content[..]) {
    //                             matches.push((search_term_match.start(), search_term_match.end()));
    //                         }
    //                         if !matches.is_empty() {
    //                             //let new_paragraph_node = Node::new(NodeData::Element {
    //                             //  name: QualName::new(None, ns!(), local_name!("p")),
    //                             //attrs: RefCell::new(vec![]),
    //                             // template_contents: RefCell::new(None),
    //                             // mathml_annotation_xml_integration_point: false,
    //                             //});
    //                             let mut index: usize = 0;
    //                             for (start, end) in matches.iter() {
    //                                 dom.append(
    //                                     &new_paragraph_node,
    //                                     AppendNode(Node::new(NodeData::Text {
    //                                         contents: RefCell::new(
    //                                             search_content[index..*start].into(),
    //                                         ),
    //                                     })),
    //                                 );
    //                                 let new_mark_node_text = Node::new(NodeData::Text {
    //                                     contents: RefCell::new(search_content[*start..*end].into()),
    //                                 });
    //                                 let new_mark_node = Node::new(NodeData::Element {
    //                                     name: QualName::new(None, ns!(), local_name!("mark")),
    //                                     attrs: RefCell::new(vec![]),
    //                                     template_contents: RefCell::new(None),
    //                                     mathml_annotation_xml_integration_point: false,
    //                                 });
    //                                 dom.append(
    //                                     &new_mark_node,
    //                                     NodeOrText::AppendNode(new_mark_node_text),
    //                                 );
    //                                 dom.append(
    //                                     &new_paragraph_node,
    //                                     NodeOrText::AppendNode(new_mark_node),
    //                                 );
    //                                 index = *end;
    //                             }
    //                             dom.append(
    //                                 &new_paragraph_node,
    //                                 AppendNode(Node::new(NodeData::Text {
    //                                     contents: RefCell::new(search_content[index..].into()),
    //                                 })),
    //                             );
    //                         }
    //                     }
    //                 }
    //                 if new_paragraph_node.children.borrow().is_empty() {
    //                     return None;
    //                 } else {
    //                     return Some(new_paragraph_node);
    //                 }
    //             }
    //         }
    //         None
    //     }
    //
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

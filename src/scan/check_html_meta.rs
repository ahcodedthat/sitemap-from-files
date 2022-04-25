use html5ever::tendril::TendrilSink as _;
use markup5ever::{
	expanded_name,
	local_name,
	namespace_url,
	ns,
};
use std::{
	borrow::Cow,
	cell::{Cell, RefCell},
	io::{self, Read},
	rc::Rc,
};

#[cfg(debug_assertions)]
use std::{
	mem,
	rc::Weak as RcWeak,
};

pub struct HtmlMeta {
	pub no_index: bool,
	#[cfg(debug_assertions)]
	all_nodes: Vec<RcWeak<Node>>,
}

impl HtmlMeta {
	pub fn read(input: &mut impl Read) -> io::Result<Self> {
		let mut opts = html5ever::ParseOpts::default();
		opts.tree_builder.scripting_enabled = false;

		#[cfg_attr(not(debug_assertions), allow(unused_mut))]
		let mut result =
			html5ever::parse_document(HtmlSink::new(), opts)
			.from_utf8()
			.read_from(input)?;

		#[cfg(debug_assertions)] {
			// In debug builds, make sure we free all our memory.
			for node in mem::take(&mut result.all_nodes) {
			if let Some(node) = RcWeak::upgrade(&node) {
				panic!(
					"node not freed ({} strong references): {node:#?}",
					Rc::strong_count(&node) - 1,
				);
			}}
		}

		Ok(result)
	}
}

#[derive(Debug)]
struct Node {
	parent: RefCell<Option<Rc<Node>>>,
	kind: NodeKind,
}

impl Node {
	fn new(kind: NodeKind) -> Rc<Self> {
		Rc::new(Self {
			parent: RefCell::new(None),
			kind,
		})
	}

	fn parent(&self) -> Option<Rc<Node>> {
		self.parent.borrow()
		.as_ref()
		.map(Rc::clone)
	}

	fn set_attr(&self, attr: &html5ever::Attribute) {
		if let Some(element) = self.as_element() {
		if element.name.expanded() == expanded_name!(html "meta") {
			let name = attr.name.expanded();
			let value = attr.value.as_ref();

			if name == expanded_name!("", "name") {
				element.is_meta_name_robots.set(value == "robots");
			}
			else if name == expanded_name!("", "content") {
				element.is_meta_content_noindex.set(
					value.split(',')
					.any(|part| part.trim().eq_ignore_ascii_case("noindex"))
				);
			}
		}}
	}

	fn as_element(&self) -> Option<&Element> {
		match &self.kind {
			NodeKind::Element(element) => Some(element),
			_ => None,
		}
	}
}

#[derive(Debug)]
enum NodeKind {
	Element(Element),
	Document,
	Pi,
	Comment,
	DocumentFragment,
}

#[derive(Debug)]
struct Element {
	name: html5ever::QualName,
	is_meta_name_robots: Cell<bool>,
	is_meta_content_noindex: Cell<bool>,
}

#[derive(Debug)]
struct HtmlSink {
	document: Rc<Node>,
	meta_elements: Vec<Rc<Node>>,
	#[cfg(debug_assertions)]
	all_nodes: Vec<RcWeak<Node>>,
}

impl HtmlSink {
	fn new() -> Self {
		let document = Node::new(NodeKind::Document);

		Self {
			#[cfg(debug_assertions)]
			all_nodes: Vec::from([
				Rc::downgrade(&document),
			]),
			document,
			meta_elements: Vec::new(),
		}
	}

	fn new_node(&mut self, node: Rc<Node>) -> Rc<Node> {
		#[cfg(debug_assertions)] {
			self.all_nodes.push(Rc::downgrade(&node));
		}

		node
	}
}

impl html5ever::interface::TreeSink for HtmlSink {
	type Handle = Rc<Node>;
	type Output = HtmlMeta;

	fn finish(self) -> Self::Output {
		let mut result = HtmlMeta {
			no_index: false,
			#[cfg(debug_assertions)]
			all_nodes: self.all_nodes,
		};

		// Good *grief*, look at this mess. `if let … else` (or proper list comprehensions, like in Scala) would be *so* much nicer here.

		// For every node that we've collected so far that is a `<meta>` element…
		for node in self.meta_elements {
			// Make sure it really is an HTML `<meta>` element. This should always check out, so panic if it doesn't.
			let element = node.as_element().expect("`self.meta_elements` contains a non-element");
			assert_eq!(element.name.expanded(), expanded_name!(html "meta"), "`self.meta_elements` contains an element that is not HTML `<meta>`");

			// Is this `<meta name=robots content=noindex>`?
			if element.is_meta_name_robots.get() && element.is_meta_content_noindex.get() {
			// Does it have a parent?
			if let Some(ancestor_head) = node.parent() {
			// Is the parent an element?
			if let Some(ancestor_head_element) = ancestor_head.as_element() {
			// Is the parent element an HTML `<head>`?
			if ancestor_head_element.name.expanded() == expanded_name!(html "head") {
			// Does `<head>` have a parent?
			if let Some(ancestor_html) = ancestor_head.parent() {
			// Is `<head>`'s parent an element?
			if let Some(ancestor_html_element) = ancestor_html.as_element() {
			// Is `<head>`'s parent element `<html>`?
			if ancestor_html_element.name.expanded() == expanded_name!(html "html") {
			// Does `<html>` have a parent?
			if let Some(ancestor_document) = ancestor_html.parent() {
			// Is `<html>`'s parent the document?
			if Rc::ptr_eq(&ancestor_document, &self.document) {
				result.no_index = true;
			}}}}}}}}}
		}

		result
	}

	fn parse_error(&mut self, _: Cow<'static, str>) {}

	fn get_document(&mut self) -> Self::Handle {
		Rc::clone(&self.document)
	}

	fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> html5ever::ExpandedName<'a> {
		target.as_element()
		.expect("`TreeSink::elem_name` called on a non-element")
		.name.expanded()
	}

	fn create_element(
		&mut self,
		name: html5ever::QualName,
		attrs: Vec<html5ever::Attribute>,
		_flags: html5ever::tree_builder::ElementFlags,
	) -> Self::Handle {
		let is_meta: bool = name.expanded() == expanded_name!(html "meta");

		let element = self.new_node(Node::new(NodeKind::Element(Element {
			name,
			is_meta_name_robots: Cell::new(false),
			is_meta_content_noindex: Cell::new(false),
		})));

		for attr in &attrs {
			element.set_attr(attr);
		}

		if is_meta {
			self.meta_elements.push(Rc::clone(&element));
		}

		element
	}

	fn create_comment(&mut self, _text: html5ever::tendril::StrTendril) -> Self::Handle {
		self.new_node(Node::new(NodeKind::Comment))
	}

	fn create_pi(&mut self, _target: html5ever::tendril::StrTendril, _data: html5ever::tendril::StrTendril) -> Self::Handle {
		self.new_node(Node::new(NodeKind::Pi))
	}

	fn append(
		&mut self,
		parent: &Self::Handle,
		child: html5ever::tree_builder::NodeOrText<Self::Handle>,
	) {
		if let html5ever::tree_builder::NodeOrText::AppendNode(node) = child {
			node.parent.replace(Some(Rc::clone(parent)));
		}
	}

	fn append_based_on_parent_node(
		&mut self,
		element: &Self::Handle,
		prev_element: &Self::Handle,
		child: html5ever::tree_builder::NodeOrText<Self::Handle>,
	) {
		if element.parent().is_some() {
			self.append_before_sibling(element, child);
		}
		else {
			self.append(prev_element, child);
		}
	}

	fn append_doctype_to_document(
		&mut self,
		_name: html5ever::tendril::StrTendril,
		_public_id: html5ever::tendril::StrTendril,
		_system_id: html5ever::tendril::StrTendril,
	) {}

	fn get_template_contents(&mut self, _target: &Self::Handle) -> Self::Handle {
		self.new_node(Node::new(NodeKind::DocumentFragment))
	}

	fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
		Rc::ptr_eq(x, y)
	}

	fn set_quirks_mode(&mut self, _mode: html5ever::tree_builder::QuirksMode) {}

	fn append_before_sibling(&mut self, sibling: &Self::Handle, new_node: html5ever::tree_builder::NodeOrText<Self::Handle>) {
		self.append(
			&sibling.parent().expect("`append_before_sibling` given a `sibling` that has no parent"),
			new_node,
		);
	}

	fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
		for attr in &attrs {
			target.set_attr(attr);
		}
	}

	fn remove_from_parent(&mut self, target: &Self::Handle) {
		target.parent.replace(None);
	}

	fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
		node.parent.replace(Some(Rc::clone(new_parent)));
	}
}

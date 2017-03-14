#![deny(missing_docs)]

//! Defines the [`Polytope<V>`](struct.Polytope.html) data structure and related methods for
//! constructing [polytopes](https://en.wikipedia.org/wiki/Polytope).

macro_rules! boxed {
    [$($x:expr),*] => (Box::new([$($x),*]));
}

macro_rules! collect {
    ($x:expr, $y:ty) => ($x.collect::<Vec<$y>>().into_boxed_slice());
}

/// An element represented as the list of indices of its lower dimensional bounding elements.
pub type Element = Box<[usize]>;

/// A data structure that represents a [polytope](https://en.wikipedia.org/wiki/Polytope) — a flat
/// sided multi-dimensional object where the vertices are connected by edges, edges by faces, faces
/// by cells, etc.
///
/// Vertex type is represented by the type parameter `V` and it must be implemented by the client
/// depending on the problem at hand. For geometrical applications, it would make sense to represent
/// a vertex as an _n_-dimensional vector, where _n_ is equal to the dimension of the polytope, but
/// this representation is not forced by the library.
///
/// The rest of the elements (edges, faces, cells, etc.) are represented as the list of indices of
/// the lower dimensional bounding elements. For example, an edge bounded by the 4th and 5th
/// vertices is represented as `[4, 5]`.
///
/// # Examples
///
/// The following code creates a 0-dimensional polytope (_i.e._ a point).
///
/// ```
/// use polytope::Polytope;
/// let point = Polytope::<String>::new("".to_string());
/// assert_eq!(point.dimension(), 0);
/// assert_eq!(point.vertices().len(), 1);
/// ```
///
/// Given a point, one can create a line, a rectangle and a prism using
/// [`extrude()`](#method.extrude):
///
/// ```
/// use polytope::Polytope;
/// let pull_in = |v: &String| v.clone() + "-";
/// let push_out = |v: &String| v.clone() + "+";
/// let point = Polytope::<String>::new("".to_string());
/// let line = point.extrude(&pull_in, &push_out);
/// let rectangle = line.extrude(&pull_in, &push_out);
/// let prism = rectangle.extrude(&pull_in, &push_out);
/// assert_eq!(prism.dimension(), 3);
/// assert_eq!(prism.vertices().len(), 8);
/// assert_eq!(prism.elements(0).len(), 12);
/// assert_eq!(prism.elements(1).len(), 6);
/// assert_eq!(prism.elements(2).len(), 1);
/// ```
///
/// ...or, given a rectangle, one can create double pyramid using [`cone()`](#method.cone):
///
/// ```
/// use polytope::Polytope;
/// let pull_in = |v: &String| v.clone() + "-";
/// let push_out = |v: &String| v.clone() + "+";
/// let point = Polytope::<String>::new("".to_string());
/// let line = point.extrude(&pull_in, &push_out);
/// let rectangle = line.extrude(&pull_in, &push_out);
/// let pyramid = rectangle.cone("0000".to_string(), pull_in, push_out);
/// assert_eq!(pyramid.dimension(), 3);
/// assert_eq!(pyramid.vertices().len(), 9);
/// assert_eq!(pyramid.elements(0).len(), 16);
/// assert_eq!(pyramid.elements(1).len(), 10);
/// assert_eq!(pyramid.elements(2).len(), 2);
/// ```
#[derive(Debug)]
pub struct Polytope<V> {
    /// List of vertices.
    vertices: Box<[V]>,

    /// List of element boundaries by dimension.
    ///
    /// For example, `elements[0]` is the list of edge boundaries, `elements[1]` is the list of face
    /// boundaries, etc.
    elements: Box<[Box<[Element]>]>,
}

impl<V> Polytope<V> {
    /// Constructs a 0-dimensional polytope with one vertex.
    pub fn new(vertex: V) -> Self {
        Polytope::<V> {
            vertices: boxed![vertex],
            elements: boxed![],
        }
    }

    /// Returns the dimension of the polytope.
    ///
    /// This is equal to the dimension of the highest dimensional element in the list of elements.
    #[inline]
    pub fn dimension(&self) -> usize {
        self.elements.len()
    }

    /// Lends the vertex list.
    #[inline]
    pub fn vertices(&self) -> &[V] {
        self.vertices.as_ref()
    }

    /// Lends the element list for the given dimension.
    #[inline]
    pub fn elements(&self, dimension: usize) -> &[Element] {
        self.elements[dimension].as_ref()
    }

    fn from_vecs(vertices: Vec<V>, elements: Vec<Vec<Element>>) -> Self {
        let mut elements = elements;

        let vertices = vertices.into_boxed_slice();
        let elements = collect!(elements.drain(..)
            .map(|mut elems| collect!(elems.drain(..), Element)), Box<[Element]>);
        Polytope::<V> {
            vertices: vertices,
            elements: elements,
        }
    }

    fn replicate_vertices<F1, F2>(&self, map_1: F1, map_2: F2, out: &mut Vec<V>)
        where F1: Fn(&V) -> V,
              F2: Fn(&V) -> V
    {
        for v in self.vertices.iter() {
            out.push(map_1(v));
            out.push(map_2(v));
        }
    }

    fn replicate_elements(&self, out: &mut Vec<Vec<Element>>) {
        for elems in self.elements.iter() {
            let mut new_elems = Vec::<Element>::new();
            for e in elems.iter() {
                new_elems.push(collect!(e.iter().map(|b| b * 2 + 0), usize));
                new_elems.push(collect!(e.iter().map(|b| b * 2 + 1), usize));
            }
            out.push(new_elems);
        }
    }

    /// Extrudes the polytope into the higher dimension.
    ///
    /// Two replicas of the polytope is created by applying the functions `pull_in` and `push_out`
    /// to the vertices. Then, the replicas are linked via higher dimensional elements (vertices via
    /// edges, edges via faces, etc.).
    ///
    /// This can be used to construct lines out of vertices, rectangle out of lines, etc.
    pub fn extrude<F1, F2>(&self, pull_in: F1, push_out: F2) -> Self
        where F1: Fn(&V) -> V,
              F2: Fn(&V) -> V
    {
        // Replicate vertices by pulling them in and pushing them out.
        let mut new_vertices = Vec::<V>::new();
        self.replicate_vertices(pull_in, push_out, &mut new_vertices);

        // Replicate the rest of the elements;
        let mut new_elements = Vec::<Vec<Element>>::new();
        self.replicate_elements(&mut new_elements);
        new_elements.push(vec![]); // make room for the new dimension

        // Link replicated vertices to each other using edges
        let mut new_edges = Vec::<Element>::new();
        for i in 0..self.vertices.len() {
            new_edges.push(boxed![i * 2 + 0, i * 2 + 1]);
        }
        let mut offset = new_elements[0].len();
        new_elements[0].extend(new_edges);

        // Link the rest of the elements using higher-dimensional elements.
        for (d, elems_this) in self.elements.iter().enumerate() {
            let mut new_elems_next = Vec::<Element>::new();
            for (i, e) in elems_this.iter().enumerate() {
                new_elems_next.push(collect!(e.iter()
                    .map(|b| offset + b)
                    .chain([i * 2 + 0, i * 2 + 1].iter().cloned()), usize));
            }
            offset = new_elements[d + 1].len();
            new_elements[d + 1].extend(new_elems_next);
        }

        Self::from_vecs(new_vertices, new_elements)
    }

    /// Constructs a higher dimensional [double cone](https://en.wikipedia.org/wiki/Cone_(geometry))
    /// out of the polytope.
    ///
    /// Two replicas of the polytope is created by applying the functions `pull_in` and `push_out`
    /// to the vertices. Then, the replicas are linked to the given apex via higher dimensional
    /// elements.
    ///
    /// This can be used to construct double triangle, double pyramid, double cone, etc.
    pub fn cone<F1, F2>(&self, apex: V, pull_in: F1, push_out: F2) -> Self
        where F1: Fn(&V) -> V,
              F2: Fn(&V) -> V
    {
        // Replicate vertices by pulling them in and pushing them out.
        let mut new_vertices = Vec::<V>::new();
        self.replicate_vertices(pull_in, push_out, &mut new_vertices);

        // Replicate the rest of the elements;
        let mut new_elements = Vec::<Vec<Element>>::new();
        self.replicate_elements(&mut new_elements);
        new_elements.push(vec![]); // make room for the new dimension

        // Add the apex.
        let apex_index = new_vertices.len();
        new_vertices.push(apex);

        // Link replicated vertices to the apex using edges.
        let mut new_edges = Vec::<Element>::new();
        for i in 0..self.vertices.len() {
            new_edges.push(boxed![i * 2 + 0, apex_index]);
            new_edges.push(boxed![i * 2 + 1, apex_index]);
        }
        let mut offset = new_elements[0].len();
        new_elements[0].extend(new_edges);

        // Link the rest of the elements using higher-dimensional elements.
        for (d, elems_this) in self.elements.iter().enumerate() {
            let mut new_elems_next = Vec::<Element>::new();
            for (i, e) in elems_this.iter().enumerate() {
                new_elems_next.push(collect!(e.iter()
                    .map(|b| offset + b * 2 + 0)
                    .chain([i * 2 + 0].iter().cloned()), usize));
                new_elems_next.push(collect!(e.iter()
                    .map(|b| offset + b * 2 + 1)
                    .chain([i * 2 + 1].iter().cloned()), usize));
            }
            offset = new_elements[d + 1].len();
            new_elements[d + 1].extend(new_elems_next);
        }

        Self::from_vecs(new_vertices, new_elements)
    }
}

#[cfg(test)]
mod tests {
    use ::Polytope;

    #[derive(Debug)]
    struct MyVertex {
        coords: Box<[f64]>,
    }

    impl Default for MyVertex {
        fn default() -> Self {
            MyVertex { coords: Box::new([]) }
        }
    }

    impl MyVertex {
        fn promote(&self, h: f64) -> Self {
            MyVertex { coords: collect!(self.coords.iter().chain([h].iter()).map(|&x| x), f64) }
        }
    }

    #[test]
    fn polytope_new() {
        let p = Polytope::<MyVertex>::new(Default::default());
        assert_eq!(p.vertices.len(), 1);
        assert_eq!(p.elements.len(), 0);
    }

    #[test]
    fn extrude_point() {
        let p = Polytope::<MyVertex>::new(Default::default());
        let q = p.extrude(|v| v.promote(-1.0),
                          |v| v.promote( 1.0));
        assert_eq!(q.vertices.len(), 2);
        assert!(q.vertices[0].coords == Box::new([-1.0]));
        assert!(q.vertices[1].coords == Box::new([ 1.0]));
        assert_eq!(q.elements.len(), 1);
        assert!(q.elements[0] == Box::new([Box::new([0, 1])]));
    }

    #[test]
    fn extrude_line() {
        let p = Polytope::<MyVertex>::new(Default::default());
        let p = p.extrude(|v| v.promote(-1.0),
                          |v| v.promote( 1.0));
        let q = p.extrude(|v| v.promote(-2.0),
                          |v| v.promote( 2.0));
        assert_eq!(q.vertices.len(), 4);
        assert!(q.vertices[0].coords == Box::new([-1.0, -2.0]));
        assert!(q.vertices[1].coords == Box::new([-1.0,  2.0]));
        assert!(q.vertices[2].coords == Box::new([ 1.0, -2.0]));
        assert!(q.vertices[3].coords == Box::new([ 1.0,  2.0]));
        assert_eq!(q.elements.len(), 2);
        assert!(q.elements[0] == Box::new([
            Box::new([0, 2]),
            Box::new([1, 3]),
            Box::new([0, 1]),
            Box::new([2, 3]),
        ]));
        assert!(q.elements[1] == Box::new([Box::new([2, 3, 0, 1])]));
    }

    #[test]
    fn cone_line() {
        let p = Polytope::<MyVertex>::new(Default::default());
        let p = p.extrude(|v| v.promote(-1.0),
                          |v| v.promote(1.0));
        let q = p.cone(MyVertex { coords: boxed![0.0, 0.0] },
                       |v| v.promote(-2.0),
                       |v| v.promote(2.0));
        assert_eq!(q.vertices.len(), 5);
        assert!(q.vertices[0].coords == Box::new([-1.0, -2.0]));
        assert!(q.vertices[1].coords == Box::new([-1.0,  2.0]));
        assert!(q.vertices[2].coords == Box::new([ 1.0, -2.0]));
        assert!(q.vertices[3].coords == Box::new([ 1.0,  2.0]));
        assert!(q.vertices[4].coords == Box::new([ 0.0,  0.0]));
        assert_eq!(q.elements.len(), 2);
        assert!(q.elements[0] == Box::new([
            Box::new([0, 2]),
            Box::new([1, 3]),
            Box::new([0, 4]),
            Box::new([1, 4]),
            Box::new([2, 4]),
            Box::new([3, 4]),
        ]));
        assert!(q.elements[1] == Box::new([
            Box::new([2, 4, 0]),
            Box::new([3, 5, 1]),
        ]));
    }

    #[test]
    fn extrude_cone_line() {
        let p = Polytope::<MyVertex>::new(Default::default());
        let p = p.extrude(|v| v.promote(-1.0),
                          |v| v.promote( 1.0));
        let p = p.cone(MyVertex { coords: boxed![0.0, 0.0] },
                       |v| v.promote(-2.0),
                       |v| v.promote( 2.0));
        let q = p.extrude(|v| v.promote(-3.0), |v| v.promote(3.0));
        assert_eq!(q.vertices.len(), 10);
        assert!(q.vertices[0].coords == Box::new([-1.0, -2.0, -3.0]));
        assert!(q.vertices[1].coords == Box::new([-1.0, -2.0,  3.0]));
        assert!(q.vertices[2].coords == Box::new([-1.0,  2.0, -3.0]));
        assert!(q.vertices[3].coords == Box::new([-1.0,  2.0,  3.0]));
        assert!(q.vertices[4].coords == Box::new([ 1.0, -2.0, -3.0]));
        assert!(q.vertices[5].coords == Box::new([ 1.0, -2.0,  3.0]));
        assert!(q.vertices[6].coords == Box::new([ 1.0,  2.0, -3.0]));
        assert!(q.vertices[7].coords == Box::new([ 1.0,  2.0,  3.0]));
        assert!(q.vertices[8].coords == Box::new([ 0.0,  0.0, -3.0]));
        assert!(q.vertices[9].coords == Box::new([ 0.0,  0.0,  3.0]));
        assert_eq!(q.elements.len(), 3);
        assert!(q.elements[0] == Box::new([
            Box::new([0, 4]),
            Box::new([1, 5]),
            Box::new([2, 6]),
            Box::new([3, 7]),
            Box::new([0, 8]),
            Box::new([1, 9]),
            Box::new([2, 8]),
            Box::new([3, 9]),
            Box::new([4, 8]),
            Box::new([5, 9]),
            Box::new([6, 8]),
            Box::new([7, 9]),
            Box::new([0, 1]),
            Box::new([2, 3]),
            Box::new([4, 5]),
            Box::new([6, 7]),
            Box::new([8, 9]),
        ]));
        assert!(q.elements[1] == Box::new([
            Box::new([4, 8, 0]),
            Box::new([5, 9, 1]),
            Box::new([6, 10, 2]),
            Box::new([7, 11, 3]),
            Box::new([12, 14, 0, 1]),
            Box::new([13, 15, 2, 3]),
            Box::new([12, 16, 4, 5]),
            Box::new([13, 16, 6, 7]),
            Box::new([14, 16, 8, 9]),
            Box::new([15, 16, 10, 11]),
        ]));
        assert!(q.elements[2] == Box::new([
            Box::new([6, 8, 4, 0, 1]),
            Box::new([7, 9, 5, 2, 3]),
        ]));
    }
}

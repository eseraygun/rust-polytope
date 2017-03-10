use std::mem;

macro_rules! boxed {
    [$($x:expr),*] => (Box::new([$($x),*]));
}

macro_rules! collect {
    ($x:expr, $y:ty) => ($x.collect::<Vec<$y>>().into_boxed_slice());
}

type Element = Box<[usize]>;

pub struct Polytope<V> {
    dimensions: usize,
    vertices: Box<[V]>,
    elements: Box<[Box<[Element]>]>,
}

impl<V: Default> Polytope<V> {
    pub fn new() -> Self {
        Polytope::<V> {
            dimensions: 0,
            vertices: boxed![V::default()],
            elements: boxed![],
        }
    }
}

impl<V> Polytope<V> {
    pub fn extrude<F1, F2>(&self, pull_in: F1, push_out: F2) -> Self
            where F1: Fn(&V) -> V,
                  F2: Fn(&V) -> V {
        // Vertices are a special case: Create a promoted pair of each and linked the pairs.
        let mut new_vertices = Vec::<V>::new();
        let mut new_edges = Vec::<Element>::new();
        for v in self.vertices.iter() {
            // Pulled in and pushed out copies.
            let i = new_vertices.len();
            new_vertices.push(pull_in(v));
            let j = new_vertices.len();
            new_vertices.push(push_out(v));

            // Linkage element.
            new_edges.push(boxed![i, j]);
        }

        // For the rest of the dimensions, create a pair of each element and linked them via a
        // higher dimensional element.
        let mut new_elements = Vec::<Vec<Element>>::new();
        let mut new_elems_this = new_edges;
        let mut new_elems_next = Vec::<Element>::new();
        for elems_this in self.elements.iter() {
            for e in elems_this.iter() {
                // Pulled in and pushed out copies.
                let i = new_elems_this.len();
                new_elems_this.push(collect!(e.iter().map(|b| b * 2 + 0), usize));
                let j = new_elems_this.len();
                new_elems_this.push(collect!(e.iter().map(|b| b * 2 + 1), usize));

                // Linkage element.
                new_elems_next.push(collect!(e.iter().chain([i, j].iter()).cloned(), usize));
            }
            // Reference-safe way of doing
            //   new_elements.push(new_elems_this);
            //   new_elems_this = new_elems_next;
            //   new_elems_next = vec![];
            new_elements.push(
                mem::replace(&mut new_elems_this,
                             mem::replace(&mut new_elems_next, vec![])));
        }
        // Don't forget the final dimension.
        new_elements.push(new_elems_this);

        let new_vertices = new_vertices.into_boxed_slice();
        let new_elements = collect!(new_elements.drain(..).map(
            |mut elements_of_this_dimension|
            collect!(elements_of_this_dimension.drain(..), Element)
        ), Box<[Element]>);
        Polytope::<V> {
            dimensions: self.dimensions + 1,
            vertices: new_vertices,
            elements: new_elements,
        }
    }
}

#[cfg(test)]
mod tests {
    use element::*;

    struct MyVertex {
        coords: Box<[f64]>,
    }

    impl Default for MyVertex {
        fn default() -> Self {
            MyVertex {
                coords: Box::new([]),
            }
        }
    }

    impl MyVertex {
        fn promote(&self, h: f64) -> Self {
            MyVertex {
                coords: collect!(self.coords.iter().chain([h].iter()).map(|&x| x), f64),
            }
        }
    }

    #[test]
    fn polytope_new() {
        let p = Polytope::<MyVertex>::new();
        assert_eq!(p.dimensions, 0);
        assert_eq!(p.vertices.len(), 1);
        assert_eq!(p.elements.len(), 0);
    }

    #[test]
    fn extrude_point() {
        let p = Polytope::<MyVertex>::new();
        let q = p.extrude(|v| v.promote(-1.0), |v| v.promote(1.0));
        assert_eq!(q.dimensions, 1);
        assert_eq!(q.vertices.len(), 2);
        assert!(q.vertices[0].coords == Box::new([-1.0]));
        assert!(q.vertices[1].coords == Box::new([ 1.0]));
        assert_eq!(q.elements.len(), 1);
        assert!(q.elements[0] == Box::new([Box::new([0, 1])]));
    }

    #[test]
    fn extrude_line() {
        let p = Polytope::<MyVertex>::new();
        let p = p.extrude(|v| v.promote(-1.0), |v| v.promote(1.0));
        let q = p.extrude(|v| v.promote(-2.0), |v| v.promote(2.0));
        assert_eq!(q.dimensions, 2);
        assert_eq!(q.vertices.len(), 4);
        assert!(q.vertices[0].coords == Box::new([-1.0, -2.0]));
        assert!(q.vertices[1].coords == Box::new([-1.0,  2.0]));
        assert!(q.vertices[2].coords == Box::new([ 1.0, -2.0]));
        assert!(q.vertices[3].coords == Box::new([ 1.0,  2.0]));
        assert_eq!(q.elements.len(), 2);
        assert!(q.elements[0] == Box::new([
            Box::new([0, 1]),
            Box::new([2, 3]),
            Box::new([0, 2]),
            Box::new([1, 3]),
        ]));
        assert!(q.elements[1] == Box::new([Box::new([0, 1, 2, 3])]));
    }
}
macro_rules! boxed {
    [$($x:expr),*] => (Box::new([$($x),*]));
}

macro_rules! collect {
    ($x:expr, $y:ty) => ($x.collect::<Vec<$y>>().into_boxed_slice());
}

type Element = Box<[usize]>;

#[derive(Debug)]
pub struct Polytope<V> {
    vertices: Box<[V]>,
    elements: Box<[Box<[Element]>]>,
}

impl<V> Polytope<V> {
    pub fn new(dimensionless: V) -> Self {
        Polytope::<V> {
            vertices: boxed![dimensionless],
            elements: boxed![],
        }
    }

    pub fn dimensions(&self) -> usize {
        self.elements.len()
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

    pub fn cone<F1, F2>(&self, tip: V, pull_in: F1, push_out: F2) -> Self
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

        // Add the tip vertex.
        let tip_index = new_vertices.len();
        new_vertices.push(tip);

        // Link replicated vertices to the tip vertex using edges.
        let mut new_edges = Vec::<Element>::new();
        for i in 0..self.vertices.len() {
            new_edges.push(boxed![i * 2 + 0, tip_index]);
            new_edges.push(boxed![i * 2 + 1, tip_index]);
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
    use element::*;

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

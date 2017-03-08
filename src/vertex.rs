use vector::Vector;

pub struct Vertex<F: Clone> {
    coords: Vector<F>,
}

impl<F: Clone> Vertex<F> {
    pub fn new_trivial() -> Vertex<F> {
        Vertex::<F> {
            coords: Vec::new(),
        }
    }

    pub fn new(coords: &Vector<F>) -> Vertex<F> {
        Vertex::<F> {
            coords: coords.clone(),
        }
    }

    pub fn dimension(&self) -> usize {
        self.coords.len()
    }
}

#[cfg(test)]
mod tests {
    use vertex::*;

    #[test]
    fn new_trivial_vertex() {
        let v = Vertex::<f32>::new_trivial();
        assert!(v.dimension() == 0);
    }

    #[test]
    fn new_vertex() {
        let v = Vertex::<f32>::new(&[1.0, 2.0, 3.0].to_vec());
        assert!(v.dimension() == 3);
        assert!(v.coords[0] == 1.0);
        assert!(v.coords[1] == 2.0);
        assert!(v.coords[2] == 3.0);
    }
}

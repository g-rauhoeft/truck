use crate::topo_traits::*;
use std::collections::HashMap;
use truck_topology::*;

/// A trait for a unified definition of the function `mapped`.

impl<P, C, S> Mapped<P, C, S> for Vertex<P> {
    /// Returns a new vertex whose point is mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::topo_traits::Mapped;
    /// let v0 = Vertex::new(1);
    /// let v1 = v0.mapped(
    ///     &move |i: &usize| *i + 1,
    ///     &<()>::clone,
    ///     &<()>::clone,
    /// );
    /// assert_eq!(*v1.lock_point().unwrap(), 2);
    /// ```
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        _: &FC,
        _: &FS,
    ) -> Self {
        Vertex::new(point_mapping(&*self.lock_point().unwrap()))
    }
}

impl<P, C, S> Mapped<P, C, S> for Edge<P, C> {
    /// Returns a new edge whose curve is mapped by `curve_mapping` and
    /// whose end points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::topo_traits::Mapped;
    /// let v0 = Vertex::new(0);
    /// let v1 = Vertex::new(1);
    /// let edge0 = Edge::new(&v0, &v1, 2);
    /// let edge1 = edge0.mapped(
    ///     &move |i: &usize| *i + 10,
    ///     &move |j: &usize| *j + 20,
    ///     &<()>::clone,
    /// );
    ///
    /// assert_eq!(*edge1.front().lock_point().unwrap(), 10);
    /// assert_eq!(*edge1.back().lock_point().unwrap(), 11);
    /// assert_eq!(*edge1.lock_curve().unwrap(), 22);
    /// ```
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        let v0 = self
            .absolute_front()
            .mapped(point_mapping, curve_mapping, surface_mapping);
        let v1 = self
            .absolute_back()
            .mapped(point_mapping, curve_mapping, surface_mapping);
        let curve = curve_mapping(&*self.lock_curve().unwrap());
        let mut edge = Edge::debug_new(&v0, &v1, curve);
        if edge.orientation() != self.orientation() {
            edge.invert();
        }
        edge
    }
}

impl<P, C, S> Mapped<P, C, S> for Wire<P, C> {
    /// Returns a new wire whose curves are mapped by `curve_mapping` and
    /// whose points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::topo_traits::Mapped;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4]);
    /// let wire0: Wire<usize, usize> = vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[2], &v[1], 110).inverse(),
    ///     Edge::new(&v[3], &v[4], 120),
    ///     Edge::new(&v[4], &v[0], 130),
    /// ].into();
    /// let wire1 = wire0.mapped(
    ///     &move |i: &usize| *i + 10,
    ///     &move |j: &usize| *j + 1000,
    ///     &<()>::clone,
    /// );
    ///
    /// // Check the points
    /// for (v0, v1) in wire0.vertex_iter().zip(wire1.vertex_iter()) {
    ///     let i = *v0.lock_point().unwrap();
    ///     let j = *v1.lock_point().unwrap();
    ///     assert_eq!(i + 10, j);
    /// }
    ///
    /// // Check the curves and orientation
    /// for (edge0, edge1) in wire0.edge_iter().zip(wire1.edge_iter()) {
    ///     let i = *edge0.lock_curve().unwrap();
    ///     let j = *edge1.lock_curve().unwrap();
    ///     assert_eq!(i + 1000, j);
    ///     assert_eq!(edge0.orientation(), edge1.orientation());
    /// }
    ///
    /// // Check the connection
    /// assert_eq!(wire1[0].back(), wire1[1].front());
    /// assert_ne!(wire1[1].back(), wire1[2].front());
    /// assert_eq!(wire1[2].back(), wire1[3].front());
    /// assert_eq!(wire1[3].back(), wire1[0].front());
    /// ```
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        let mut vertex_map: HashMap<VertexID<P>, Vertex<P>> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v.id()).is_none() {
                let vert = v.mapped(point_mapping, curve_mapping, surface_mapping);
                vertex_map.insert(v.id(), vert);
            }
        }
        let mut wire = Wire::new();
        let mut edge_map: HashMap<EdgeID<C>, Edge<P, C>> = HashMap::new();
        for edge in self.edge_iter() {
            if let Some(new_edge) = edge_map.get(&edge.id()) {
                if edge.absolute_front() == edge.front() {
                    wire.push_back(new_edge.clone());
                } else {
                    wire.push_back(new_edge.inverse());
                }
            } else {
                let vertex0 = vertex_map.get(&edge.absolute_front().id()).unwrap().clone();
                let vertex1 = vertex_map.get(&edge.absolute_back().id()).unwrap().clone();
                let curve = curve_mapping(&*edge.lock_curve().unwrap());
                let new_edge = Edge::debug_new(&vertex0, &vertex1, curve);
                if edge.orientation() {
                    wire.push_back(new_edge.clone());
                } else {
                    wire.push_back(new_edge.inverse());
                }
                edge_map.insert(edge.id(), new_edge);
            }
        }
        wire
    }
}

impl<P, C, S> Mapped<P, C, S> for Face<P, C, S> {
    /// Returns a new face whose surface is mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::topo_traits::Mapped;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4, 5, 6]);
    /// let wire0 = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[1], &v[2], 200),
    ///     Edge::new(&v[2], &v[3], 300),
    ///     Edge::new(&v[3], &v[0], 400),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     Edge::new(&v[4], &v[5], 500),
    ///     Edge::new(&v[6], &v[5], 600).inverse(),
    ///     Edge::new(&v[6], &v[4], 700),
    /// ]);
    /// let face0 = Face::new(vec![wire0, wire1], 10000);
    /// let face1 = face0.mapped(
    ///     &move |i: &usize| *i + 10,
    ///     &move |j: &usize| *j + 1000,
    ///     &move |k: &usize| *k + 100000,
    /// );
    /// # for wire in face1.boundaries() {
    /// #    assert!(wire.is_closed());
    /// #    assert!(wire.is_simple());
    /// # }
    ///
    /// assert_eq!(
    ///     *face0.lock_surface().unwrap() + 100000,
    ///     *face1.lock_surface().unwrap(),
    /// );
    /// let biters0 = face0.boundary_iters();
    /// let biters1 = face1.boundary_iters();
    /// for (biter0, biter1) in biters0.into_iter().zip(biters1) {
    ///     for (edge0, edge1) in biter0.zip(biter1) {
    ///         assert_eq!(
    ///             *edge0.front().lock_point().unwrap() + 10,
    ///             *edge1.front().lock_point().unwrap(),
    ///         );
    ///         assert_eq!(
    ///             *edge0.back().lock_point().unwrap() + 10,
    ///             *edge1.back().lock_point().unwrap(),
    ///         );
    ///         assert_eq!(edge0.orientation(), edge1.orientation());
    ///         assert_eq!(
    ///             *edge0.lock_curve().unwrap() + 1000,
    ///             *edge1.lock_curve().unwrap(),
    ///         );
    ///     }
    /// }
    /// ```
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        let wires: Vec<_> = self
            .absolute_boundaries()
            .iter()
            .map(|wire| wire.mapped(point_mapping, curve_mapping, surface_mapping))
            .collect();
        let surface = surface_mapping(&*self.lock_surface().unwrap());
        let mut face = Face::debug_new(wires, surface);
        if !self.orientation() {
            face.invert();
        }
        face
    }
}

impl<P, C, S> Mapped<P, C, S> for Shell<P, C, S> {
    /// Returns a new shell whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_modeling::topo_traits::Mapped;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4, 5, 6]);
    /// let wire0 = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[1], &v[2], 200),
    ///     Edge::new(&v[2], &v[3], 300),
    ///     Edge::new(&v[3], &v[0], 400),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     Edge::new(&v[4], &v[5], 500),
    ///     Edge::new(&v[6], &v[5], 600).inverse(),
    ///     Edge::new(&v[6], &v[4], 700),
    /// ]);
    /// let face0 = Face::new(vec![wire0, wire1], 10000);
    /// let face1 = face0.mapped(
    ///     &move |i: &usize| *i + 7,
    ///     &move |j: &usize| *j + 700,
    ///     &move |k: &usize| *k + 10000,
    /// );
    /// let shell0 = Shell::from(vec![face0, face1.inverse()]);
    /// let shell1 = shell0.mapped(
    ///     &move |i: &usize| *i + 50,
    ///     &move |j: &usize| *j + 5000,
    ///     &move |k: &usize| *k + 500000,
    /// );
    /// # for face in shell1.face_iter() {
    /// #    for bdry in face.absolute_boundaries() {
    /// #        assert!(bdry.is_closed());
    /// #        assert!(bdry.is_simple());
    /// #    }
    /// # }
    ///
    /// for (face0, face1) in shell0.face_iter().zip(shell1.face_iter()) {
    ///     assert_eq!(
    ///         *face0.lock_surface().unwrap() + 500000,
    ///         *face1.lock_surface().unwrap(),
    ///     );
    ///     assert_eq!(face0.orientation(), face1.orientation());
    ///     let biters0 = face0.boundary_iters();
    ///     let biters1 = face1.boundary_iters();
    ///     for (biter0, biter1) in biters0.into_iter().zip(biters1) {
    ///         for (edge0, edge1) in biter0.zip(biter1) {
    ///             assert_eq!(
    ///                 *edge0.front().lock_point().unwrap() + 50,
    ///                 *edge1.front().lock_point().unwrap(),
    ///             );
    ///             assert_eq!(
    ///                 *edge0.back().lock_point().unwrap() + 50,
    ///                 *edge1.back().lock_point().unwrap(),
    ///             );
    ///             assert_eq!(
    ///                 *edge0.lock_curve().unwrap() + 5000,
    ///                 *edge1.lock_curve().unwrap(),
    ///             );
    ///         }
    ///     }
    /// }
    /// ```
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        let mut shell = Shell::new();
        let mut vmap: HashMap<VertexID<P>, Vertex<P>> = HashMap::new();
        let vertex_iter = self
            .iter()
            .flat_map(Face::absolute_boundaries)
            .flat_map(Wire::vertex_iter);
        for vertex in vertex_iter {
            if vmap.get(&vertex.id()).is_none() {
                let new_vertex = vertex.mapped(point_mapping, curve_mapping, surface_mapping);
                vmap.insert(vertex.id(), new_vertex);
            }
        }
        let mut edge_map: HashMap<EdgeID<C>, Edge<P, C>> = HashMap::new();
        for face in self.face_iter() {
            let mut wires = Vec::new();
            for biter in face.absolute_boundaries() {
                let mut wire = Wire::new();
                for edge in biter {
                    if let Some(new_edge) = edge_map.get(&edge.id()) {
                        if edge.absolute_front() == edge.front() {
                            wire.push_back(new_edge.clone());
                        } else {
                            wire.push_back(new_edge.inverse());
                        }
                    } else {
                        let v0 = vmap.get(&edge.absolute_front().id()).unwrap();
                        let v1 = vmap.get(&edge.absolute_back().id()).unwrap();
                        let curve = curve_mapping(&*edge.lock_curve().unwrap());
                        let new_edge = Edge::debug_new(v0, v1, curve);
                        if edge.orientation() {
                            wire.push_back(new_edge.clone());
                        } else {
                            wire.push_back(new_edge.inverse());
                        }
                        edge_map.insert(edge.id(), new_edge);
                    }
                }
                wires.push(wire);
            }
            let surface = surface_mapping(&*face.lock_surface().unwrap());
            let mut new_face = Face::debug_new(wires, surface);
            if !face.orientation() {
                new_face.invert();
            }
            shell.push(new_face);
        }
        shell
    }
}

impl<P, C, S> Mapped<P, C, S> for Solid<P, C, S> {
    /// Returns a new solid whose surfaces are mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    #[inline(always)]
    fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
        &self,
        point_mapping: &FP,
        curve_mapping: &FC,
        surface_mapping: &FS,
    ) -> Self {
        Solid::debug_new(
            self.boundaries()
                .iter()
                .map(|shell| shell.mapped(point_mapping, curve_mapping, surface_mapping))
                .collect(),
        )
    }
}

#[cfg(test)]
mod invert_variation {
    use super::*;

    #[test]
    fn invert_mapped_edge() {
        let v0 = Vertex::new(0);
        let v1 = Vertex::new(1);
        let edge0 = Edge::new(&v0, &v1, 2).inverse();
        let edge1 = edge0.mapped(
            &move |i: &usize| *i + 10,
            &move |j: &usize| *j + 20,
            &<()>::clone,
        );
        assert_eq!(*edge1.absolute_front().lock_point().unwrap(), 10);
        assert_eq!(*edge1.absolute_back().lock_point().unwrap(), 11);
        assert_eq!(edge0.orientation(), edge1.orientation());
        assert_eq!(*edge1.lock_curve().unwrap(), 22);
    }

    #[test]
    fn invert_mapped_face() {
        let v = Vertex::news(&[0, 1, 2, 3, 4, 5, 6]);
        let wire0 = Wire::from(vec![
            Edge::new(&v[0], &v[1], 100),
            Edge::new(&v[1], &v[2], 200),
            Edge::new(&v[2], &v[3], 300),
            Edge::new(&v[3], &v[0], 400),
        ]);
        let wire1 = Wire::from(vec![
            Edge::new(&v[4], &v[5], 500),
            Edge::new(&v[6], &v[5], 600).inverse(),
            Edge::new(&v[6], &v[4], 700),
        ]);
        let face0 = Face::new(vec![wire0, wire1], 10000).inverse();
        let face1 = face0.mapped(
            &move |i: &usize| *i + 10,
            &move |j: &usize| *j + 1000,
            &move |k: &usize| *k + 100000,
        );

        assert_eq!(
            *face0.lock_surface().unwrap() + 100000,
            *face1.lock_surface().unwrap(),
        );
        assert_eq!(face0.orientation(), face1.orientation());
        let biters0 = face0.boundary_iters();
        let biters1 = face1.boundary_iters();
        for (biter0, biter1) in biters0.into_iter().zip(biters1) {
            for (edge0, edge1) in biter0.zip(biter1) {
                assert_eq!(
                    *edge0.front().lock_point().unwrap() + 10,
                    *edge1.front().lock_point().unwrap(),
                );
                assert_eq!(
                    *edge0.back().lock_point().unwrap() + 10,
                    *edge1.back().lock_point().unwrap(),
                );
                assert_eq!(
                    *edge0.lock_curve().unwrap() + 1000,
                    *edge1.lock_curve().unwrap(),
                );
            }
        }
    }
}

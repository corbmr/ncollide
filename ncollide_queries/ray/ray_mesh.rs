use std::ops::Index;
use na::{Pnt2, Vec3, Transform, Rotate, Norm};
use na;
use ray::{Ray, LocalRayCast, RayCast, RayIntersection};
use ray;
use entities::shape::{BaseMesh, BaseMeshElement, TriMesh, Polyline};
use entities::bounding_volume::AABB;
use entities::partitioning::BVTCostFn;
use math::{Scalar, Point, Vect};


impl< P, I, E> LocalRayCast<P> for BaseMesh<P, I, E>
    where P: Point,
          I: Index<usize, Output = usize>,
          E: BaseMeshElement<I, P> + LocalRayCast<P> {
    #[inline]
    fn toi_with_ray(&self, ray: &Ray<P>, _: bool) -> Option<<P::Vect as Vect>::Scalar> {
        let mut cost_fn = BaseMeshRayToiCostFn { mesh: self, ray: ray };

        self.bvt().best_first_search(&mut cost_fn).map(|(_, res)| res)
    }

    #[inline]
    fn toi_and_normal_with_ray(&self, ray: &Ray<P>, _: bool) -> Option<RayIntersection<P::Vect>> {
        let mut cost_fn = BaseMeshRayToiAndNormalCostFn { mesh: self, ray: ray };

        self.bvt().best_first_search(&mut cost_fn).map(|(_, res)| res)
    }

    fn toi_and_normal_and_uv_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<RayIntersection<P::Vect>> {
        if self.uvs().is_none() || na::dim::<P>() != 3 {
            return self.toi_and_normal_with_ray(ray, solid);
        }

        let mut cost_fn = BaseMeshRayToiAndNormalAndUVsCostFn { mesh: self, ray: ray };
        let cast = self.bvt().best_first_search(&mut cost_fn);

        match cast {
            None                => None,
            Some((best, inter)) => {
                let toi = inter.0.toi;
                let n   = inter.0.normal.clone();
                let uv  = inter.1; // barycentric coordinates to compute the exact uvs.

                let idx   = &self.indices()[*best];
                let uvs   = self.uvs().as_ref().unwrap();

                let uv1 = uvs[idx[0]];
                let uv2 = uvs[idx[1]];
                let uv3 = uvs[idx[2]];

                let uvx = uv1.x * uv.x + uv2.x * uv.y + uv3.x * uv.z;
                let uvy = uv1.y * uv.x + uv2.y * uv.y + uv3.y * uv.z;

                // XXX: this interpolation should be done on the two other ray cast too!
                match *self.normals() {
                    None         => {
                        Some(RayIntersection::new_with_uvs(toi, n, Some(Pnt2::new(uvx, uvy))))
                    },
                    Some(ref ns) => {
                        let n1 = &ns[idx[0]];
                        let n2 = &ns[idx[1]];
                        let n3 = &ns[idx[2]];

                        let mut n123 = *n1 * uv.x + *n2 * uv.y + *n3 * uv.z;

                        if na::is_zero(&n123.normalize_mut()) {
                            Some(RayIntersection::new_with_uvs(toi, n, Some(Pnt2::new(uvx, uvy))))
                        }
                        else {
                            if na::dot(&n123, &ray.dir) > na::zero() {
                                Some(RayIntersection::new_with_uvs(toi, -n123, Some(Pnt2::new(uvx, uvy))))
                            }
                            else {
                                Some(RayIntersection::new_with_uvs(toi, n123, Some(Pnt2::new(uvx, uvy))))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<P, M, I, E> RayCast<P, M> for BaseMesh<P, I, E>
    where P: Point,
          M: Transform<P> + Rotate<P::Vect>,
          I: Index<usize, Output = usize>,
          E: BaseMeshElement<I, P> + LocalRayCast<P> {
}


/*
 * Costs functions.
 */
struct BaseMeshRayToiCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh:  &'a BaseMesh<P, I, E>,
    ray:   &'a Ray<P>
}

impl<'a, P, I, E> BVTCostFn<<P::Vect as Vect>::Scalar, usize, AABB<P>, <P::Vect as Vect>::Scalar>
for BaseMeshRayToiCostFn<'a, P, I, E>
    where P: Point,
          E: BaseMeshElement<I, P> + LocalRayCast<P> {
    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<P>) -> Option<<P::Vect as Vect>::Scalar> {
        aabb.toi_with_ray(self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(<P::Vect as Vect>::Scalar, <P::Vect as Vect>::Scalar)> {
        self.mesh.element_at(*b).toi_with_ray(self.ray, true).map(|toi| (toi, toi))
    }
}

struct BaseMeshRayToiAndNormalCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh:  &'a BaseMesh<P, I, E>,
    ray:   &'a Ray<P>
}

impl<'a, P, I, E> BVTCostFn<<P::Vect as Vect>::Scalar, usize, AABB<P>, RayIntersection<P::Vect>>
for BaseMeshRayToiAndNormalCostFn<'a, P, I, E>
    where P: Point,
          E: BaseMeshElement<I, P> + LocalRayCast<P> {
    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<P>) -> Option<<P::Vect as Vect>::Scalar> {
        aabb.toi_with_ray(self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(<P::Vect as Vect>::Scalar, RayIntersection<P::Vect>)> {
        self.mesh.element_at(*b).toi_and_normal_with_ray(self.ray, true).map(|inter| (inter.toi, inter))
    }
}

struct BaseMeshRayToiAndNormalAndUVsCostFn<'a, P: 'a + Point, I: 'a, E: 'a> {
    mesh:  &'a BaseMesh<P, I, E>,
    ray:   &'a Ray<P>
}

impl<'a, P, I, E> BVTCostFn<<P::Vect as Vect>::Scalar, usize, AABB<P>, (RayIntersection<P::Vect>, Vec3<<P::Vect as Vect>::Scalar>)>
for BaseMeshRayToiAndNormalAndUVsCostFn<'a, P, I, E>
    where P: Point,
          I: Index<usize, Output = usize>,
          E: BaseMeshElement<I, P> + LocalRayCast<P> {
    #[inline]
    fn compute_bv_cost(&mut self, aabb: &AABB<P>) -> Option<<P::Vect as Vect>::Scalar> {
        aabb.toi_with_ray(self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize)
        -> Option<(<P::Vect as Vect>::Scalar, (RayIntersection<P::Vect>, Vec3<<P::Vect as Vect>::Scalar>))> {
        let vs  = &self.mesh.vertices()[..];
        let idx = &self.mesh.indices()[*b];

        let a = &vs[idx[0]];
        let b = &vs[idx[1]];
        let c = &vs[idx[2]];

        ray::triangle_ray_intersection(a, b, c, self.ray).map(|inter| (inter.0.toi.clone(), inter))
    }
}

/*
 * fwd impls. to the exact shapes.
 */
impl<P> LocalRayCast<P> for TriMesh<P>
    where P: Point {
    #[inline]
    fn toi_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<<P::Vect as Vect>::Scalar> {
        self.base_mesh().toi_with_ray(ray, solid)
    }

    #[inline]
    fn toi_and_normal_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<RayIntersection<P::Vect>> {
        self.base_mesh().toi_and_normal_with_ray(ray, solid)
    }

    #[inline]
    fn toi_and_normal_and_uv_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<RayIntersection<P::Vect>> {
        self.base_mesh().toi_and_normal_and_uv_with_ray(ray, solid)
    }
}

impl<P, M> RayCast<P, M> for TriMesh<P>
    where P: Point,
          M: Transform<P> + Rotate<P::Vect> {
}

impl<P> LocalRayCast<P> for Polyline<P>
    where P: Point {
    #[inline]
    fn toi_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<<P::Vect as Vect>::Scalar> {
        self.base_mesh().toi_with_ray(ray, solid)
    }

    #[inline]
    fn toi_and_normal_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<RayIntersection<P::Vect>> {
        self.base_mesh().toi_and_normal_with_ray(ray, solid)
    }

    #[inline]
    fn toi_and_normal_and_uv_with_ray(&self, ray: &Ray<P>, solid: bool) -> Option<RayIntersection<P::Vect>> {
        self.base_mesh().toi_and_normal_and_uv_with_ray(ray, solid)
    }
}

impl<P, M> RayCast<P, M> for Polyline<P>
    where P: Point,
          M: Transform<P> + Rotate<P::Vect> {
}

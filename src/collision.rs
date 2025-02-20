// let radius = f32x32::splat(1.0); // Particle radius, adjust as needed           
// // Calculate the distance between all pairs of particles
// for i in 0..COUNT / SIMD_LEVEL {
//     for j in i + 1..COUNT / SIMD_LEVEL {
//         // Calculate the distance between particle i and particle j
//         let dx = x[i] - x[j]; // Difference in x position
//         let dy = y[i] - y[j]; // Difference in y position
        
//         let distance_squared = dx * dx + dy * dy; // Squared distance between particles
//         let radius_sum = radius + radius; // Sum of radii
//         let radius_sum_squared = radius_sum * radius_sum;
        
//         let collision_threshold = radius_sum * radius_sum; // Compare squared distance with squared radius sum

//         let colliding = distance_squared.simd_lt(collision_threshold); // Check if particles are colliding
        
//         // Handle the collision if colliding
//         if colliding.any() {
    
//         }
//     }
// }    
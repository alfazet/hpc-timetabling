#include "executor/adjuster.cuh"

void Stats::update(usize cur_generation, kernels::Penalty cur_penalty) {
    usize delta_gen = cur_generation - generation;
    if (min_penalty == kernels::MAX_PENALTY || cur_penalty < min_penalty) {
        progress += delta_gen;
        stagnation = 0;
        min_penalty = cur_penalty;
    } else {
        progress = 0;
        stagnation += delta_gen;
    }
    generation = cur_generation;
}

void Stats::print(f32 mut_rate, f32 cross_rate, f32 elites_frac, f32 worst_frac) const {
    printf("\n");
    printf("min penalty after %lu generations: ", generation);
    min_penalty.print();
    printf("\n");
    if (stagnation > 0) {
        printf("stagnating for %lu generations\n", stagnation);
    } else {
        printf("progressing for %lu generations\n", progress);
    }
    printf("mutation rate: %.4f, crossover rate: %.4f\n", mut_rate, cross_rate);
    printf("elites: %.4f%%, anti-elites: %.4f%%", elites_frac * 100, worst_frac * 100);
    printf("\n");
}

Adjuster::Adjuster(f32 delta, f32 min_mut, f32 max_mut, f32 min_cross, f32 max_cross, f32 min_elites_frac,
                   f32 max_elites_frac, f32 min_worst_frac, f32 max_worst_frac)
    : delta(delta), min_mut(min_mut), max_mut(max_mut), min_cross(min_cross), max_cross(max_cross),
      min_elites_frac(min_elites_frac), max_elites_frac(max_elites_frac), min_worst_frac(min_worst_frac),
      max_worst_frac(max_worst_frac) {}

void Adjuster::adjust(const Stats &stats, kernels::Mutation &mut, kernels::Crossover &cross,
                      kernels::Population &population) const {
    if (stats.stagnation > 0) {
        f32 scale = log2f(1.0f + static_cast<f32>(stats.stagnation));
        mut.prob = fminf(mut.prob + delta * scale, max_mut);
        cross.prob = fmaxf(cross.prob - delta * scale, min_cross);
        population.elites_frac = fmaxf(population.elites_frac - delta * scale, min_elites_frac);
        population.worst_frac = fminf(population.worst_frac + delta * scale, max_worst_frac);
    } else if (stats.progress > 0) {
        mut.prob = fmaxf(mut.prob - delta, min_mut);
        cross.prob = fminf(cross.prob + delta, max_cross);
        population.elites_frac = fminf(population.elites_frac + delta, max_elites_frac);
        population.worst_frac = fmaxf(population.worst_frac - delta, min_worst_frac);
    }
}
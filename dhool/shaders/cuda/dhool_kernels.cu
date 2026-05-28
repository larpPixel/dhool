#include <cuda_runtime.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>

extern "C" {

struct DhoolCudaDeviceInfo {
    int available;
    char name[256];
    int driver_version;
    int runtime_version;
    unsigned long long global_memory_bytes;
    int multiprocessor_count;
    int error_code;
    char error_message[512];
};

struct DhoolRenderReport {
    int status;
    float gpu_ms;
    float readback_ms;
    int error_code;
    char error_message[512];
};

struct DhoolRenderParams {
    uint32_t width;
    uint32_t height;
    uint32_t frame_index;
    uint32_t total_frames;
    uint64_t seed;
    float time_seconds;
    float duration_seconds;
    float fps;
    uint32_t quality;
    uint32_t field_width;
    uint32_t field_height;
    uint32_t substeps;
    float velocity_decay;
    float vorticity;
    float buoyancy;
    float turbulence;
    float swirl;
    float advection_dissipation;
    float plasma_emission_strength;
    float plasma_filament_density;
    float plasma_core_temperature;
    float plasma_edge_wrapping;
    uint32_t particles_count;
    float particles_size_min;
    float particles_size_max;
    float particles_lifetime;
    float particles_flow_coupling;
    float particles_ember_brightness;
    float smoke_density;
    float smoke_scattering;
    float smoke_absorption;
    float smoke_color_cool[3];
    float smoke_color_warm[3];
    float bloom_threshold;
    float bloom_strength;
    float bloom_radius;
    uint32_t bloom_pyramid_levels;
    float grade_exposure;
    float grade_contrast;
    float grade_saturation;
    float grade_black_point;
    float grade_vignette;
    float grade_grain;
};

}

struct BloomView {
    const float4* levels[8];
    int widths[8];
    int heights[8];
    int level_count;
};

static_assert(sizeof(unsigned long long) == sizeof(uint64_t), "unexpected uint64 size");

__device__ float clamp01(float v) {
    return fminf(1.0f, fmaxf(0.0f, v));
}

__device__ float saturate(float v) {
    return clamp01(v);
}

__device__ float2 add2(float2 a, float2 b) {
    return make_float2(a.x + b.x, a.y + b.y);
}

__device__ float2 sub2(float2 a, float2 b) {
    return make_float2(a.x - b.x, a.y - b.y);
}

__device__ float2 mul2(float2 a, float s) {
    return make_float2(a.x * s, a.y * s);
}

__device__ float dot2(float2 a, float2 b) {
    return a.x * b.x + a.y * b.y;
}

__device__ float len2(float2 a) {
    return sqrtf(dot2(a, a));
}

__device__ float2 normalize2(float2 v) {
    float l = len2(v);
    return l > 1.0e-6f ? make_float2(v.x / l, v.y / l) : make_float2(0.0f, 1.0f);
}

__device__ float3 add3(float3 a, float3 b) {
    return make_float3(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ float3 mul3(float3 a, float s) {
    return make_float3(a.x * s, a.y * s, a.z * s);
}

__device__ float3 mix3(float3 a, float3 b, float t) {
    return make_float3(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t
    );
}

__device__ uint64_t splitmix64(uint64_t x) {
    x += 0x9e3779b97f4a7c15ULL;
    uint64_t z = x;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

__device__ float hash_u64_to_unit(uint64_t x) {
    return static_cast<float>((splitmix64(x) >> 40) & 0xFFFFFFULL) / 16777216.0f;
}

__device__ float hash_i2(int x, int y, uint64_t seed) {
    uint64_t h = static_cast<uint64_t>(x) * 0x9e3779b185ebca87ULL;
    h ^= static_cast<uint64_t>(y) * 0xc2b2ae3d27d4eb4fULL;
    h ^= seed;
    return hash_u64_to_unit(h);
}

__device__ float smooth(float t) {
    return t * t * (3.0f - 2.0f * t);
}

__device__ float noise2(float2 p, uint64_t seed) {
    int ix = static_cast<int>(floorf(p.x));
    int iy = static_cast<int>(floorf(p.y));
    float fx = p.x - floorf(p.x);
    float fy = p.y - floorf(p.y);
    float ux = smooth(fx);
    float uy = smooth(fy);
    float a = hash_i2(ix, iy, seed);
    float b = hash_i2(ix + 1, iy, seed);
    float c = hash_i2(ix, iy + 1, seed);
    float d = hash_i2(ix + 1, iy + 1, seed);
    float ab = a + (b - a) * ux;
    float cd = c + (d - c) * ux;
    return ab + (cd - ab) * uy;
}

__device__ float fbm(float2 p, uint64_t seed) {
    float sum = 0.0f;
    float amp = 0.52f;
    float freq = 1.0f;
    for (int i = 0; i < 5; ++i) {
        sum += amp * noise2(make_float2(p.x * freq, p.y * freq), seed + static_cast<uint64_t>(i) * 131ULL);
        freq *= 2.03f;
        amp *= 0.49f;
        p = make_float2(p.x * 1.17f + p.y * 0.23f, -p.x * 0.19f + p.y * 1.11f);
    }
    return sum;
}

__device__ float ridge(float v) {
    return 1.0f - fabsf(v * 2.0f - 1.0f);
}

__device__ float2 field_flow(float2 p, float t, uint64_t seed, const DhoolRenderParams& params) {
    float n1 = fbm(make_float2(p.x * 1.6f + t * 0.10f, p.y * 1.4f - t * 0.05f), seed + 17ULL);
    float n2 = fbm(make_float2(p.x * 2.4f - t * 0.07f, p.y * 2.1f + t * 0.09f), seed + 53ULL);
    float angle = (n1 * 2.0f - 1.0f) * 6.2831853f * (0.65f + params.turbulence * 0.30f);
    angle += (p.x * 1.7f - p.y * 0.8f + n2) * params.swirl;
    float2 curl = make_float2(cosf(angle), sinf(angle));
    float2 lift = make_float2(0.18f * sinf(p.y * 5.0f + t), 0.72f + params.buoyancy);
    float2 orbital = normalize2(make_float2(-p.y - 0.14f, p.x + 0.18f));
    return add2(add2(mul2(curl, params.vorticity), mul2(lift, 0.42f)), mul2(orbital, 0.25f * params.swirl));
}

__device__ float plasma_mask(float2 p, float t, uint64_t seed, const DhoolRenderParams& params) {
    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 flow = field_flow(p, t, seed, params);
    float2 warped = add2(p, mul2(flow, 0.045f + params.plasma_edge_wrapping * 0.025f));
    warped.x += (fbm(make_float2(p.x * 2.0f, p.y * 2.0f + t * 0.15f), seed + 71ULL) - 0.5f) * 0.11f;
    warped.y += (fbm(make_float2(p.x * 2.8f - t * 0.2f, p.y * 2.0f), seed + 73ULL) - 0.5f) * 0.09f;

    float x = warped.x / aspect;
    float y = warped.y;
    float left_curve = 0.025f - 0.47f * x + 0.045f * sinf(11.0f * x + t * 0.55f);
    float right_curve = -0.060f + 0.66f * (x - 0.04f) + 0.060f * sinf(9.0f * x - t * 0.45f);
    float lower_curve = -0.315f + 0.34f * (x + 0.08f) + 0.040f * sinf(13.0f * x + t * 0.38f);
    left_curve += (fbm(make_float2(x * 5.0f + t * 0.12f, 1.3f), seed + 81ULL) - 0.5f) * 0.050f;
    right_curve += (fbm(make_float2(x * 5.5f - t * 0.10f, 4.7f), seed + 83ULL) - 0.5f) * 0.060f;
    lower_curve += (fbm(make_float2(x * 4.8f + t * 0.08f, 8.2f), seed + 85ULL) - 0.5f) * 0.045f;

    float left_gate = smooth(saturate((0.16f - x) * 3.4f));
    float right_gate = smooth(saturate((x + 0.03f) * 3.5f));
    float lower_gate = smooth(saturate((x + 0.18f) * 3.2f)) * smooth(saturate((-0.03f - y) * 3.0f));
    float d_left = fabsf(y - left_curve);
    float d_right = fabsf(y - right_curve);
    float d_lower = fabsf(y - lower_curve);
    float core_width = 23.0f + params.plasma_filament_density * 10.0f;
    float sheath_width = 7.5f + params.plasma_filament_density * 1.5f;
    float core = expf(-d_left * core_width) * left_gate;
    core += expf(-d_right * core_width) * right_gate;
    core += expf(-d_lower * core_width) * lower_gate;
    float sheath = expf(-d_left * sheath_width) * left_gate;
    sheath += expf(-d_right * sheath_width) * right_gate;
    sheath += expf(-d_lower * sheath_width) * lower_gate;

    float filament_wave = sinf((warped.x * 11.0f + warped.y * 7.0f + t * 0.82f) * (1.3f + params.plasma_filament_density));
    filament_wave += 0.7f * sinf((warped.x * -15.0f + warped.y * 9.0f - t * 0.46f) * (1.1f + params.plasma_filament_density * 0.6f));
    float filament = powf(saturate(ridge(0.5f + 0.24f * filament_wave)), 6.2f);
    float fine = powf(saturate(fbm(make_float2(warped.x * 18.0f + t, warped.y * 18.0f - t * 0.6f), seed + 101ULL)), 3.0f);
    float void_mask = expf(-(x * x * 18.0f + (y - 0.18f) * (y - 0.18f) * 34.0f));
    float source = core * (0.85f + filament * fine * 1.8f) + sheath * 0.16f;
    return source * (1.0f - 0.90f * void_mask);
}

__device__ float terrain_ridge_y(float2 p, float t, uint64_t seed) {
    float left_mass = 0.11f * expf(-((p.x + 0.62f) * (p.x + 0.62f)) / 0.055f);
    float right_mass = 0.18f * expf(-((p.x - 0.56f) * (p.x - 0.56f)) / 0.070f);
    float center_dip = -0.10f * expf(-(p.x * p.x) / 0.18f);
    float rock_noise = 0.055f * fbm(make_float2(p.x * 4.0f + t * 0.02f, 0.5f), seed + 223ULL);
    return -0.405f + left_mass + right_mass + center_dip + 0.035f * sinf(p.x * 9.0f + 1.1f) + rock_noise;
}

__device__ float terrain_mask(float2 p, float t, uint64_t seed) {
    float ridge_line = terrain_ridge_y(p, t, seed);
    return smooth(saturate((ridge_line - p.y) * 18.0f));
}

__device__ float terrain_rim(float2 p, float t, uint64_t seed) {
    float ridge_line = terrain_ridge_y(p, t, seed);
    return expf(-fabsf(p.y - ridge_line) * 42.0f) * smooth(saturate((p.y - ridge_line + 0.08f) * 9.0f));
}

__device__ float smoke_layer(float2 p, float t, uint64_t seed, const DhoolRenderParams& params, float plasma) {
    float2 flow = field_flow(p, t, seed + 303ULL, params);
    float2 advected = add2(p, mul2(flow, -0.20f * params.advection_dissipation));
    advected.y -= t * 0.055f;
    float broad = fbm(make_float2(advected.x * 1.15f, advected.y * 1.05f), seed + 307ULL);
    float sheet = fbm(make_float2(advected.x * 3.1f + broad * 0.8f, advected.y * 2.2f - t * 0.08f), seed + 311ULL);
    float haze = saturate((broad * 0.66f + sheet * 0.34f - 0.39f) * 1.45f);
    float depth = smooth(saturate((p.y + 0.58f) * 1.7f)) * smooth(saturate((0.58f - p.y) * 1.2f));
    float plasma_cut = 1.0f - saturate(plasma * 0.28f);
    return haze * depth * plasma_cut * params.smoke_density;
}

__device__ float particle_layer(
    float2 uv,
    float2 p,
    float t,
    uint64_t seed,
    const DhoolRenderParams& params,
    float aspect
) {
    float requested = sqrtf(fmaxf(static_cast<float>(params.particles_count), 1.0f));
    float cells_x = fminf(240.0f, fmaxf(56.0f, requested * 0.42f));
    float cells_y = cells_x / aspect;
    float2 q = make_float2(uv.x * cells_x, uv.y * cells_y);
    int cx = static_cast<int>(floorf(q.x));
    int cy = static_cast<int>(floorf(q.y));
    float acc = 0.0f;
    for (int oy = -1; oy <= 1; ++oy) {
        for (int ox = -1; ox <= 1; ++ox) {
            int gx = cx + ox;
            int gy = cy + oy;
            uint64_t cell_seed = seed ^ (static_cast<uint64_t>(gx) * 0x632be59bd9b4e019ULL) ^ (static_cast<uint64_t>(gy) * 0x9e3779b97f4a7c15ULL);
            float hx = hash_u64_to_unit(cell_seed + 11ULL);
            float hy = hash_u64_to_unit(cell_seed + 19ULL);
            float heat = hash_u64_to_unit(cell_seed + 29ULL);
            float phase = hash_u64_to_unit(cell_seed + 41ULL);
            float age = fmodf(phase + t / fmaxf(params.particles_lifetime, 0.01f), 1.0f);
            float2 pos_uv = make_float2((static_cast<float>(gx) + hx) / cells_x, (static_cast<float>(gy) + hy) / cells_y);
            float2 pos_p = make_float2((pos_uv.x - 0.5f) * aspect, 0.5f - pos_uv.y);
            float spawn = plasma_mask(pos_p, t - age * 0.32f, seed + 997ULL, params);
            if (spawn < 0.06f && heat < 0.72f) {
                continue;
            }
            float2 flow = field_flow(pos_p, t - age * 0.45f, seed + 991ULL, params);
            pos_uv.x += flow.x * params.particles_flow_coupling * age * 0.035f / aspect;
            pos_uv.y -= flow.y * params.particles_flow_coupling * age * 0.035f;
            float2 d = make_float2(uv.x - pos_uv.x, uv.y - pos_uv.y);
            float2 dir = normalize2(make_float2(flow.x / aspect, -flow.y));
            float along = dot2(d, dir);
            float cross = dot2(d, make_float2(-dir.y, dir.x));
            float size_px = params.particles_size_min + (params.particles_size_max - params.particles_size_min) * heat;
            float radius = size_px / 1080.0f;
            float streak = expf(-(cross * cross) / (radius * radius * 1.35f) - (along * along) / (radius * radius * 7.5f));
            float life = smooth(saturate(age * 5.0f)) * smooth(saturate((1.0f - age) * 2.8f));
            acc += streak * life * (0.16f + heat * 0.72f) * saturate(spawn * 1.55f);
        }
    }
    return acc * params.particles_ember_brightness;
}

__global__ void render_hdr_kernel(float4* hdr, DhoolRenderParams params) {
    uint32_t x = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= params.width || y >= params.height) {
        return;
    }

    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 uv = make_float2((static_cast<float>(x) + 0.5f) / params.width, (static_cast<float>(y) + 0.5f) / params.height);
    float2 p = make_float2((uv.x - 0.5f) * aspect, 0.5f - uv.y);
    float t = params.time_seconds;
    uint64_t seed = params.seed + static_cast<uint64_t>(params.frame_index) * 17ULL;

    float plasma = plasma_mask(p, t, seed, params);
    float smoke = smoke_layer(p, t, seed, params, plasma);
    float embers = particle_layer(uv, p, t, seed, params, aspect);
    float terrain = terrain_mask(p, t, seed);
    float rim = terrain_rim(p, t, seed);

    float central_void = expf(-((p.x * p.x) * 3.0f + (p.y - 0.15f) * (p.y - 0.15f) * 7.0f));
    float edge_focus = 0.72f + 0.28f * smooth(saturate((fabsf(p.x) - 0.10f) * 2.0f));
    float hot = powf(saturate(plasma), 1.18f) * params.plasma_emission_strength * (0.20f + params.plasma_core_temperature * 0.22f);
    float3 hot_core = make_float3(4.6f, 1.45f, 0.20f);
    float3 hot_edge = make_float3(1.8f, 0.32f, 0.055f);
    float3 ember_color = make_float3(3.3f, 0.88f, 0.16f);
    float3 cool_smoke = make_float3(params.smoke_color_cool[0], params.smoke_color_cool[1], params.smoke_color_cool[2]);
    float3 warm_smoke = make_float3(params.smoke_color_warm[0], params.smoke_color_warm[1], params.smoke_color_warm[2]);
    float3 smoke_color = mix3(cool_smoke, warm_smoke, saturate(plasma * params.smoke_scattering * 0.75f + 0.15f));
    float3 background = make_float3(0.004f, 0.0045f, 0.0055f);

    float3 color = background;
    color = add3(color, mul3(smoke_color, smoke * (0.55f + params.smoke_scattering * 0.50f)));
    color = add3(color, mul3(mix3(hot_edge, hot_core, saturate(plasma * 1.4f)), hot));
    color = add3(color, mul3(ember_color, embers * 0.44f));
    color = mul3(color, edge_focus * (1.0f - central_void * 0.58f));
    color = mix3(color, mul3(color, 0.035f), terrain);
    float rock = terrain * powf(fbm(make_float2(p.x * 8.0f + t * 0.01f, p.y * 8.0f), seed + 401ULL), 2.2f);
    color = add3(color, mul3(make_float3(0.060f, 0.046f, 0.036f), rock * 0.55f));
    color = add3(color, mul3(make_float3(1.7f, 0.48f, 0.07f), rim * (0.10f + plasma * 0.30f)));
    color = add3(color, mul3(make_float3(0.75f, 0.24f, 0.045f), plasma * smoke * params.smoke_absorption * 0.18f));

    uint32_t index = y * params.width + x;
    hdr[index] = make_float4(color.x, color.y, color.z, 1.0f);
}

__global__ void threshold_downsample_kernel(
    const float4* src,
    float4* dst,
    int src_w,
    int src_h,
    int dst_w,
    int dst_h,
    DhoolRenderParams params
) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= dst_w || y >= dst_h) {
        return;
    }
    float3 sum = make_float3(0.0f, 0.0f, 0.0f);
    for (int oy = 0; oy < 2; ++oy) {
        for (int ox = 0; ox < 2; ++ox) {
            int sx = min(src_w - 1, x * 2 + ox);
            int sy = min(src_h - 1, y * 2 + oy);
            float4 c = src[sy * src_w + sx];
            float lum = c.x * 0.2126f + c.y * 0.7152f + c.z * 0.0722f;
            float k = saturate((lum - params.bloom_threshold) / fmaxf(lum, 1.0e-4f));
            sum.x += c.x * k;
            sum.y += c.y * k;
            sum.z += c.z * k;
        }
    }
    dst[y * dst_w + x] = make_float4(sum.x * 0.25f, sum.y * 0.25f, sum.z * 0.25f, 1.0f);
}

__global__ void downsample_kernel(const float4* src, float4* dst, int src_w, int src_h, int dst_w, int dst_h) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= dst_w || y >= dst_h) {
        return;
    }
    float3 sum = make_float3(0.0f, 0.0f, 0.0f);
    for (int oy = 0; oy < 2; ++oy) {
        for (int ox = 0; ox < 2; ++ox) {
            int sx = min(src_w - 1, x * 2 + ox);
            int sy = min(src_h - 1, y * 2 + oy);
            float4 c = src[sy * src_w + sx];
            sum.x += c.x;
            sum.y += c.y;
            sum.z += c.z;
        }
    }
    dst[y * dst_w + x] = make_float4(sum.x * 0.25f, sum.y * 0.25f, sum.z * 0.25f, 1.0f);
}

__device__ float3 sample_bloom_level(const float4* src, int w, int h, float2 uv) {
    float px = saturate(uv.x) * static_cast<float>(w - 1);
    float py = saturate(uv.y) * static_cast<float>(h - 1);
    int x0 = static_cast<int>(floorf(px));
    int y0 = static_cast<int>(floorf(py));
    int x1 = min(w - 1, x0 + 1);
    int y1 = min(h - 1, y0 + 1);
    float tx = px - floorf(px);
    float ty = py - floorf(py);
    float4 a = src[y0 * w + x0];
    float4 b = src[y0 * w + x1];
    float4 c = src[y1 * w + x0];
    float4 d = src[y1 * w + x1];
    float3 ab = make_float3(a.x + (b.x - a.x) * tx, a.y + (b.y - a.y) * tx, a.z + (b.z - a.z) * tx);
    float3 cd = make_float3(c.x + (d.x - c.x) * tx, c.y + (d.y - c.y) * tx, c.z + (d.z - c.z) * tx);
    return make_float3(ab.x + (cd.x - ab.x) * ty, ab.y + (cd.y - ab.y) * ty, ab.z + (cd.z - ab.z) * ty);
}

__device__ float3 aces_tonemap(float3 x) {
    const float a = 2.51f;
    const float b = 0.03f;
    const float c = 2.43f;
    const float d = 0.59f;
    const float e = 0.14f;
    return make_float3(
        saturate((x.x * (a * x.x + b)) / (x.x * (c * x.x + d) + e)),
        saturate((x.y * (a * x.y + b)) / (x.y * (c * x.y + d) + e)),
        saturate((x.z * (a * x.z + b)) / (x.z * (c * x.z + d) + e))
    );
}

__global__ void final_grade_kernel(const float4* hdr, uchar4* out, BloomView bloom, DhoolRenderParams params) {
    uint32_t x = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= params.width || y >= params.height) {
        return;
    }
    uint32_t index = y * params.width + x;
    float2 uv = make_float2((static_cast<float>(x) + 0.5f) / params.width, (static_cast<float>(y) + 0.5f) / params.height);
    float4 base4 = hdr[index];
    float3 color = make_float3(base4.x, base4.y, base4.z);
    float radius_scale = 0.0015f + params.bloom_radius * 0.0025f;
    float3 bloom_sum = make_float3(0.0f, 0.0f, 0.0f);
    float weight_sum = 0.0f;
    for (int i = 0; i < bloom.level_count; ++i) {
        float spread = static_cast<float>(i + 1) * radius_scale;
        float weight = powf(0.74f, static_cast<float>(i));
        float3 center = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], uv);
        float3 east = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x + spread, uv.y));
        float3 west = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x - spread, uv.y));
        float3 north = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x, uv.y - spread));
        float3 south = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x, uv.y + spread));
        float3 level = mul3(add3(add3(add3(add3(center, east), west), north), south), 0.2f);
        bloom_sum = add3(bloom_sum, mul3(level, weight));
        weight_sum += weight;
    }
    if (weight_sum > 0.0f) {
        color = add3(color, mul3(bloom_sum, params.bloom_strength / weight_sum));
    }

    color = mul3(color, exp2f(params.grade_exposure));
    color = aces_tonemap(color);
    color.x = saturate((color.x - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color.y = saturate((color.y - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color.z = saturate((color.z - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color = make_float3(
        saturate((color.x - 0.5f) * params.grade_contrast + 0.5f),
        saturate((color.y - 0.5f) * params.grade_contrast + 0.5f),
        saturate((color.z - 0.5f) * params.grade_contrast + 0.5f)
    );
    float lum = color.x * 0.2126f + color.y * 0.7152f + color.z * 0.0722f;
    color = make_float3(
        saturate(lum + (color.x - lum) * params.grade_saturation),
        saturate(lum + (color.y - lum) * params.grade_saturation),
        saturate(lum + (color.z - lum) * params.grade_saturation)
    );
    float2 centered = make_float2(uv.x - 0.5f, uv.y - 0.5f);
    float vignette = 1.0f - params.grade_vignette * saturate(dot2(centered, centered) * 1.65f);
    color = mul3(color, vignette);
    float grain = (hash_u64_to_unit(params.seed ^ (static_cast<uint64_t>(x) << 32) ^ static_cast<uint64_t>(y) ^ static_cast<uint64_t>(params.frame_index) * 0x9e37ULL) - 0.5f) * params.grade_grain;
    color = make_float3(saturate(color.x + grain), saturate(color.y + grain), saturate(color.z + grain));

    out[index] = make_uchar4(
        static_cast<unsigned char>(saturate(color.x) * 255.0f + 0.5f),
        static_cast<unsigned char>(saturate(color.y) * 255.0f + 0.5f),
        static_cast<unsigned char>(saturate(color.z) * 255.0f + 0.5f),
        255
    );
}

void clear_device_info(DhoolCudaDeviceInfo* out) {
    std::memset(out, 0, sizeof(DhoolCudaDeviceInfo));
}

void clear_report(DhoolRenderReport* report) {
    std::memset(report, 0, sizeof(DhoolRenderReport));
}

void set_error(DhoolRenderReport* report, cudaError_t error, const char* context) {
    report->status = 0;
    report->error_code = static_cast<int>(error);
    std::snprintf(report->error_message, sizeof(report->error_message), "%s: %s", context, cudaGetErrorString(error));
}

extern "C" int dhool_cuda_device_info(DhoolCudaDeviceInfo* out) {
    if (out == nullptr) {
        return 0;
    }
    clear_device_info(out);
    int count = 0;
    cudaError_t err = cudaGetDeviceCount(&count);
    if (err != cudaSuccess || count <= 0) {
        out->available = 0;
        out->error_code = static_cast<int>(err);
        std::snprintf(out->error_message, sizeof(out->error_message), "cudaGetDeviceCount failed: %s", cudaGetErrorString(err));
        return 0;
    }

    cudaDeviceProp prop {};
    err = cudaGetDeviceProperties(&prop, 0);
    if (err != cudaSuccess) {
        out->available = 0;
        out->error_code = static_cast<int>(err);
        std::snprintf(out->error_message, sizeof(out->error_message), "cudaGetDeviceProperties failed: %s", cudaGetErrorString(err));
        return 0;
    }

    int driver = 0;
    int runtime = 0;
    cudaDriverGetVersion(&driver);
    cudaRuntimeGetVersion(&runtime);
    out->available = 1;
    std::snprintf(out->name, sizeof(out->name), "%s", prop.name);
    out->driver_version = driver;
    out->runtime_version = runtime;
    out->global_memory_bytes = static_cast<unsigned long long>(prop.totalGlobalMem);
    out->multiprocessor_count = prop.multiProcessorCount;
    return 1;
}

extern "C" int dhool_cuda_render_rgba8(
    const DhoolRenderParams* params,
    uint8_t* host_rgba,
    DhoolRenderReport* report
) {
    if (params == nullptr || host_rgba == nullptr || report == nullptr) {
        return 0;
    }
    clear_report(report);
    cudaError_t err = cudaSetDevice(0);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaSetDevice");
        return 0;
    }

    const size_t pixels = static_cast<size_t>(params->width) * static_cast<size_t>(params->height);
    float4* hdr = nullptr;
    uchar4* ldr = nullptr;
    float4* bloom_levels[8] {};
    int bloom_w[8] {};
    int bloom_h[8] {};
    BloomView bloom {};
    int level_count = std::max(1, std::min(8, static_cast<int>(params->bloom_pyramid_levels)));

    err = cudaMalloc(reinterpret_cast<void**>(&hdr), pixels * sizeof(float4));
    if (err != cudaSuccess) {
        set_error(report, err, "cudaMalloc hdr");
        return 0;
    }
    err = cudaMalloc(reinterpret_cast<void**>(&ldr), pixels * sizeof(uchar4));
    if (err != cudaSuccess) {
        cudaFree(hdr);
        set_error(report, err, "cudaMalloc ldr");
        return 0;
    }

    int previous_w = static_cast<int>(params->width);
    int previous_h = static_cast<int>(params->height);
    for (int i = 0; i < level_count; ++i) {
        bloom_w[i] = std::max(1, previous_w / 2);
        bloom_h[i] = std::max(1, previous_h / 2);
        err = cudaMalloc(reinterpret_cast<void**>(&bloom_levels[i]), static_cast<size_t>(bloom_w[i]) * static_cast<size_t>(bloom_h[i]) * sizeof(float4));
        if (err != cudaSuccess) {
            for (int j = 0; j < i; ++j) {
                cudaFree(bloom_levels[j]);
            }
            cudaFree(ldr);
            cudaFree(hdr);
            set_error(report, err, "cudaMalloc bloom level");
            return 0;
        }
        previous_w = bloom_w[i];
        previous_h = bloom_h[i];
    }

    cudaEvent_t gpu_start {};
    cudaEvent_t gpu_stop {};
    cudaEvent_t copy_start {};
    cudaEvent_t copy_stop {};
    cudaEventCreate(&gpu_start);
    cudaEventCreate(&gpu_stop);
    cudaEventCreate(&copy_start);
    cudaEventCreate(&copy_stop);

    dim3 block(16, 16);
    dim3 full_grid((params->width + block.x - 1) / block.x, (params->height + block.y - 1) / block.y);
    cudaEventRecord(gpu_start);
    render_hdr_kernel<<<full_grid, block>>>(hdr, *params);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        set_error(report, err, "render_hdr_kernel");
        goto cleanup;
    }

    for (int i = 0; i < level_count; ++i) {
        dim3 grid((bloom_w[i] + block.x - 1) / block.x, (bloom_h[i] + block.y - 1) / block.y);
        if (i == 0) {
            threshold_downsample_kernel<<<grid, block>>>(hdr, bloom_levels[i], params->width, params->height, bloom_w[i], bloom_h[i], *params);
        } else {
            downsample_kernel<<<grid, block>>>(bloom_levels[i - 1], bloom_levels[i], bloom_w[i - 1], bloom_h[i - 1], bloom_w[i], bloom_h[i]);
        }
        err = cudaGetLastError();
        if (err != cudaSuccess) {
            set_error(report, err, "bloom downsample kernel");
            goto cleanup;
        }
    }

    bloom.level_count = level_count;
    for (int i = 0; i < level_count; ++i) {
        bloom.levels[i] = bloom_levels[i];
        bloom.widths[i] = bloom_w[i];
        bloom.heights[i] = bloom_h[i];
    }
    final_grade_kernel<<<full_grid, block>>>(hdr, ldr, bloom, *params);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        set_error(report, err, "final_grade_kernel");
        goto cleanup;
    }
    cudaEventRecord(gpu_stop);
    err = cudaEventSynchronize(gpu_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventSynchronize gpu_stop");
        goto cleanup;
    }
    cudaEventElapsedTime(&report->gpu_ms, gpu_start, gpu_stop);

    cudaEventRecord(copy_start);
    err = cudaMemcpy(host_rgba, ldr, pixels * sizeof(uchar4), cudaMemcpyDeviceToHost);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaMemcpy readback");
        goto cleanup;
    }
    cudaEventRecord(copy_stop);
    err = cudaEventSynchronize(copy_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventSynchronize copy_stop");
        goto cleanup;
    }
    cudaEventElapsedTime(&report->readback_ms, copy_start, copy_stop);
    report->status = 1;

cleanup:
    cudaEventDestroy(gpu_start);
    cudaEventDestroy(gpu_stop);
    cudaEventDestroy(copy_start);
    cudaEventDestroy(copy_stop);
    for (int i = 0; i < level_count; ++i) {
        cudaFree(bloom_levels[i]);
    }
    cudaFree(ldr);
    cudaFree(hdr);
    return report->status;
}

#version 450

const float EPS = 1.0e-6;

layout(set = 1, binding = 0) uniform ModelMatrix {
    mat4 uniform_matrix;
};

layout(set = 1, binding = 1) uniform Material {
    vec4 albedo;
    float roughness;
    float reflectance;
    float ambient_ratio;
};

layout(set = 1, binding = 2) uniform texture2D texture_view;
layout(set = 1, binding = 3) uniform sampler texture_sampler;

layout(set = 1, binding = 4) buffer Boundary {
    vec4 boundary[];
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;
layout(location = 3) in flat uvec2 boundary_range;
layout(location = 4) in mat4 input_matrix;

layout(location = 0) out vec4 color;

float fract_distance(float x, float y) {
    float a = abs(x - y);
    float b = abs(1.0 + x - y);
    float c = abs(x - y - 1.0);
    return min(a, min(b, c));
}

vec3 current_normal() {
    if (distance(uv, vec2(position.x, (1.0 + position.y) / 2.0)) > 0.5) {
        return vec3(0.0, 0.0, 1.0);
    } else {
        return vec3(-1.0, 0.0, 1.0) / sqrt(2.0);
    }
}

uvec2 answer_range() {
    if (position.x < 0.0) {
        return uvec2(0, 4);
    } else {
        return uvec2(4, 8);
    }
}

void main() {
    if (fract_distance(fract(position.x), uv.x) > EPS) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (abs((1.0 + position.y) / 2.0 - uv.y) > EPS) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (distance(normal, current_normal()) > EPS) {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (input_matrix != uniform_matrix) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (distance(uniform_matrix[0], vec4(1.0, 2.0, 3.0, 4.0)) > EPS) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (distance(uniform_matrix[1], vec4(5.0, 6.0, 7.0, 8.0)) > EPS) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (distance(uniform_matrix[2], vec4(9.0, 10.0, 11.0, 12.0)) > EPS) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (distance(uniform_matrix[3], vec4(13.0, 14.0, 15.0, 16.0)) > EPS) {
        color = vec4(0.0, 0.0, 1.0, 1.0); 
    } else if (distance(albedo, vec4(0.2, 0.4, 0.6, 1.0)) > EPS) {
        color = vec4(1.0, 1.0, 0.0, 1.0);
    } else if (abs(roughness - 0.31415) > EPS) {
        color = vec4(1.0, 0.0, 1.0, 1.0);
    } else if (abs(reflectance - 0.29613) > EPS) {
        color = vec4(0.0, 1.0, 1.0, 1.0);
    } else if (abs(ambient_ratio - 0.92) > EPS) {
        color = vec4(0.25, 0.25, 0.25, 1.0);
    } else if (boundary_range != answer_range()) {
        color = vec4(0.5, 0.5, 0.5, 1.0);
    } else if (distance(boundary[0], vec4(0.0, 0.0, 1.0, 0.0)) > EPS) {
        color = vec4(0.75, 0.75, 0.75, 1.0);
    } else if (distance(boundary[1], vec4(1.0, 0.0, 1.0, 1.0)) > EPS) {
        color = vec4(0.75, 0.75, 0.75, 1.0);
    } else if (distance(boundary[2], vec4(1.0, 1.0, 0.0, 1.0)) > EPS) {
        color = vec4(0.75, 0.75, 0.75, 1.0);
    } else if (distance(boundary[3], vec4(0.0, 1.0, 0.0, 0.0)) > EPS) {
        color = vec4(0.75, 0.75, 0.75, 1.0);
    } else {
        vec2 tex_coord = (1.0 + vec2(position.x, -position.y)) / 2.0;
        color = texture(sampler2D(texture_view, texture_sampler), tex_coord);
    }
}

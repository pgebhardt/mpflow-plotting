#version 330

in vec3 v_position;
in vec3 v_color;
in vec4 shadow_position;

out vec4 frag_color;

uniform sampler2D shadow_map;

void main() {
    // get visibility from shadow map
    float bias = 1e-4;
    float visibility = 1.0;
    if (texture(shadow_map, shadow_position.xy / shadow_position.w).r < (shadow_position.z - bias) / shadow_position.w) {
        float visibility = 0.5;
    }

    // calculate fragment color
    frag_color = pow(vec4(visibility * v_color, 1.0), vec4(2.2, 2.2, 2.2, 1.0));
}

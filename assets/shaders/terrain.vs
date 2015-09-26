shader {
	glsl_layout float2:0
}
!!GLSL
#version 440
#pragma include <scene.glsl>

layout (std140, binding = 1) uniform ObjectData {
	float scale;
	float height_scale;
};

#ifdef _VERTEX_
layout(binding=0) uniform sampler2D heightmap;
layout(location=0) in vec2 position;
out vec2 tc;
out vec3 wPos;
out vec3 wN;
out float relHeight;
void main() {
  float height = texture(heightmap, position).r;
  vec4 wPos_ground = vec4(position.x, 0, position.y, 1.0);
  vec4 wPos_tmp = vec4(scale*wPos_ground.x, height_scale*height, scale*wPos_ground.z, 1.0);
	// TODO normal matrix
	//vec4 wN_tmp = modelMatrix * vec4(normal, 0.0);
	gl_Position = projMatrix * viewMatrix * wPos_tmp;
	tc = position;
	relHeight = height;
	wPos = wPos_tmp.xyz;
	wN = vec3(0.0);
}
#endif

#ifdef _FRAGMENT_
in vec2 tc;
in vec3 wPos;
in vec3 wN;
in float relHeight;
out vec4 color;
void main() {
	color = vec4(relHeight, relHeight, relHeight, 0.0);
}
#endif

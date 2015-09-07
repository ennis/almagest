#version 440

// uniform buffer bindings:
// 0: scene and pass global (includes lights)
// 1: per-material data
// 2: per-object data

layout (std140, binding = 0) uniform SceneData {
	mat4 viewMatrix;
	mat4 projMatrix;
	mat4 viewProjMatrix;	// = projMatrix*viewMatrix
	vec4 lightDir;
	vec4 wEye;	// in world space
	vec2 viewportSize;	// taille de la fenêtre
	vec3 wLightPos;
	vec3 lightColor;
	float lightIntensity;
};

layout (std140, binding = 1) uniform ObjectData {
	float scale;
	float height_scale;
};

// texture bindings:
// 0-3: pass textures (shadow maps, etc.)
// 4-8: per-material textures
// 8-?: per-object textures

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

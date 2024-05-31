#include <GLFW/glfw3.h>
#include <iostream>
#include <fstream>
#include <string>
#include <sstream>
#include "cloth.h"
#include <glad/glad.h>
#include <chrono>
#include <numeric>
#include <cuda_runtime.h>
#include "device_launch_parameters.h"
#include <curand_kernel.h>
#include <time.h>
#include <filesystem>
#include <optional>

extern "C" void simulate_kernel(Point * points, Spring * springs, int N, int M, int num_springs, float dt, float g, bool g_on, float m);

void key_callback(GLFWwindow* window, int key, int scancode, int action, int mode);
void framebuffer_size_callback(GLFWwindow* window, int width, int height);
void mouse_button_callback(GLFWwindow* window, int button, int action, int mode);
void cursor_position_callback(GLFWwindow* window, double xpos, double ypos);
char* read_shader_src(const std::string& path);
std::vector<Point> flatten_points(const std::vector<std::vector<Point>>& points, int N, int M);
void unflatten_points(std::vector<std::vector<Point>>& points, const std::vector<Point>& flattenedPoints, int N, int M);
void simulate_CUDA(float dt);

const int N = 40;
const int M = 40;

Cloth cloth(N, M);
double mouse_x, mouse_y;
int window_width = 1280;
int window_height = 1000;
std::optional<std::pair<int, int>> closest_point;
std::optional<std::pair<int, int>> affected_point;
std::optional<std::pair<int, int>> moving_point;

int main() {
	// initialize cloth

	float max_y = std::numeric_limits<float>::min();
	for (auto& row : cloth.points) {
		for (auto& point : row) {
			max_y = std::max(max_y, point.y);
		}
	}

	for (auto& row : cloth.points) {
		for (auto& point : row) {
			point.x -= M / 2.0f;
			point.y -= max_y;
			point.y += 30.0f;
		}
	}

	// read shader source
	std::string executable_directory = std::filesystem::path(__FILE__).parent_path().string();
	std::string vertex_shader_path = executable_directory + "/shaders/vertex.glsl";
	std::string fragment_shader_path = executable_directory + "/shaders/fragment.glsl";

	char* vertex_shader_src = read_shader_src(vertex_shader_path);
	char* fragment_shader_src = read_shader_src(fragment_shader_path);

	// Initialize GLFW
	glfwInit();

	glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
	glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
	glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

	// Create a GLFWwindow object
	GLFWwindow* window = glfwCreateWindow(1280, 1000, "Cloth Simulation", NULL, NULL);
	// Error check if the window fails to create
	if (window == NULL)
	{
		std::cout << "Failed to create GLFW window" << std::endl;
		glfwTerminate();
		return -1;
	}

	// Introduce the window into the current context
	glfwSetWindowPos(window, 600, 200);
	glfwMakeContextCurrent(window);

	//Load GLAD so it configures OpenGL
	gladLoadGL();

	// Specify the viewport of OpenGL in the Window
	glViewport(0, 0, window_width, window_height);

	// Set the callback functions
	glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);
	glfwSetKeyCallback(window, key_callback);
	glfwSetMouseButtonCallback(window, mouse_button_callback);
	glfwSetCursorPosCallback(window, cursor_position_callback);

	// Create Vertex Shader Object and get its reference
	GLuint vertexShader = glCreateShader(GL_VERTEX_SHADER);
	glShaderSource(vertexShader, 1, &vertex_shader_src, NULL);
	glCompileShader(vertexShader);

	// Create Fragment Shader Object and get its reference
	GLuint fragmentShader = glCreateShader(GL_FRAGMENT_SHADER);
	glShaderSource(fragmentShader, 1, &fragment_shader_src, NULL);
	glCompileShader(fragmentShader);

	// Create Shader Program Object and get its reference
	GLuint shaderProgram = glCreateProgram();
	glAttachShader(shaderProgram, vertexShader);
	glAttachShader(shaderProgram, fragmentShader);
	glLinkProgram(shaderProgram);

	// Delete the now useless Vertex and Fragment Shader objects
	glDeleteShader(vertexShader);
	glDeleteShader(fragmentShader);
	delete[] vertex_shader_src;
	delete[] fragment_shader_src;

	// Create reference containers for the Vartex Array Object and the Vertex Buffer Object
	GLuint VAO, VBO, EBO;

	// Generate the VAO and VBO with only 1 object each
	glGenVertexArrays(1, &VAO);
	glGenBuffers(1, &VBO);
	glGenBuffers(1, &EBO);

	// Make the VAO the current Vertex Array Object by binding it
	glBindVertexArray(VAO);

	// Bind the VBO specifying it's a GL_ARRAY_BUFFER
	glBindBuffer(GL_ARRAY_BUFFER, VBO);
	// Introduce the vertices into the VBO
	//glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

	// Configure the Vertex Attribute so that OpenGL knows how to read the VBO
	glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*)0);
	// Enable the Vertex Attribute so that OpenGL knows to use it
	glEnableVertexAttribArray(0);

	glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, EBO);

	// Bind both the VBO and VAO to 0 so that we don't accidentally modify the VAO and VBO we created
	glBindBuffer(GL_ARRAY_BUFFER, 0);
	glBindVertexArray(0);

	// Tell OpenGL which Shader Program we want to use
	glUseProgram(shaderProgram);

	// Specify the color of the background
	glClearColor(1.0f, 1.0f, 1.0f, 1.0f);

	GLint matrixLoc = glGetUniformLocation(shaderProgram, "matrix");

	std::vector<double> fps_values;
	std::vector<double> simulation_times;
	std::vector<double> draw_times;

	auto last_time = std::chrono::high_resolution_clock::now();


	// Main while loop
	while (!glfwWindowShouldClose(window))
	{
		glfwPollEvents();

		auto frame_start = std::chrono::high_resolution_clock::now();

		for (int i = 0; i < 5; i++)
		{
			auto sim_start = std::chrono::high_resolution_clock::now();
			//cloth.simulate(0.01f);
			simulate_CUDA(0.01f);
			auto sim_end = std::chrono::high_resolution_clock::now();
			double sim_time = std::chrono::duration<double, std::milli>(sim_end - sim_start).count();
			simulation_times.push_back(sim_time);
		}

		std::vector<float> vertices;
		std::vector<unsigned int> indices;

		for (auto& row : cloth.points) {
			for (auto& point : row) {
				vertices.push_back(point.x);
				vertices.push_back(point.y);
			}
		}

		for (auto& spring : cloth.springs) {
			indices.push_back(spring.p1.first * M + spring.p1.second);
			indices.push_back(spring.p2.first * M + spring.p2.second);
		}

		glBindVertexArray(VAO);
		glBindBuffer(GL_ARRAY_BUFFER, VBO);
		glBufferData(GL_ARRAY_BUFFER, vertices.size() * sizeof(float), vertices.data(), GL_STATIC_DRAW);

		glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, EBO);
		glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices.size() * sizeof(unsigned int), indices.data(), GL_STATIC_DRAW);

		// draw
		auto draw_start = std::chrono::high_resolution_clock::now();
		glClear(GL_COLOR_BUFFER_BIT);

		glDrawElements(GL_LINES, indices.size(), GL_UNSIGNED_INT, 0);

		int width, height;
		glfwGetFramebufferSize(window, &width, &height);
		float aspect_ratio = (float)width / (float)height;

		float matrix[16] = {
			0.03f / aspect_ratio, 0.0f, 0.0f, 0.0f,
			0.0f, 0.03f, 0.0f, 0.0f,
			0.0f, 0.0f, 1.0f, 0.0f,
			0.0f, 0.0f, 0.0f, 1.0f
		};

		glUniformMatrix4fv(matrixLoc, 1, GL_FALSE, matrix);


		// Swap the back buffer with the front buffer
		glfwSwapBuffers(window);
		auto draw_end = std::chrono::high_resolution_clock::now();
		double draw_time = std::chrono::duration<double, std::milli>(draw_end - draw_start).count();
		draw_times.push_back(draw_time);

		auto frame_end = std::chrono::high_resolution_clock::now();
		double frame_time = std::chrono::duration<double, std::milli>(frame_end - frame_start).count();
		double fps = 1000.0 / frame_time;
		fps_values.push_back(fps);
	}
	if (!fps_values.empty()) {
		fps_values.erase(fps_values.begin());
		double avg_fps = std::accumulate(fps_values.begin(), fps_values.end(), 0.0) / fps_values.size();
		std::cout << "Average FPS: " << avg_fps << std::endl;
	}

	if (!simulation_times.empty()) {
		double avg_sim_time = std::accumulate(simulation_times.begin(), simulation_times.end(), 0.0) / simulation_times.size();
		std::cout << "Average Simulation Time: " << avg_sim_time << " ms" << std::endl;
	}

	if (!draw_times.empty()) {
		double avg_draw_time = std::accumulate(draw_times.begin(), draw_times.end(), 0.0) / draw_times.size();
		std::cout << "Average Draw Time: " << avg_draw_time << " ms" << std::endl;
	}


	// Delete all the objects we've created
	glDeleteVertexArrays(1, &VAO);
	glDeleteBuffers(1, &VBO);
	glDeleteProgram(shaderProgram);
	// Delete window before ending the program
	glfwDestroyWindow(window);
	// Terminate GLFW before ending the program
	glfwTerminate();
	return 0;
}
void key_callback(GLFWwindow* window, int key, int scancode, int action, int mode) {
	if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
		glfwSetWindowShouldClose(window, GL_TRUE);
	}
	else if (key == GLFW_KEY_G && action == GLFW_PRESS) {
		// Toggle gravity
		cloth.g_on = !cloth.g_on;
	}
	else if (key == GLFW_KEY_F) {
		if (action == GLFW_PRESS && !affected_point) {
			int i = rand() % cloth.points.size(); 
			int j = rand() % cloth.points[0].size(); 

			cloth.points[i][j].ext_m += 5.0f;
			affected_point = std::make_pair(i, j);
		}
		else if (action == GLFW_RELEASE && affected_point) {
			auto [i, j] = *affected_point;
			cloth.points[i][j].ext_m = 0.0f;
			affected_point.reset();
		}
	}
}

void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
	glViewport(0, 0, width, height);
	window_width = width;
	window_height = height;
}

void mouse_button_callback(GLFWwindow* window, int button, int action, int mods) {
	const float max_radius = 0.025f;

	float norm_mouse_x = (mouse_x / window_width) * 2.0f - 1.0f;
	float norm_mouse_y = -((mouse_y / window_height) * 2.0f - 1.0f);

	float aspect_ratio = (float)window_width / (float)window_height;
	norm_mouse_x *= aspect_ratio;

	if (button == GLFW_MOUSE_BUTTON_RIGHT) {
		if (action == GLFW_PRESS) {
			// find closest point to mouse
			closest_point = std::nullopt;
			float min_dist = std::numeric_limits<float>::max();

			for (int i = 0; i < cloth.points.size(); i++) {
				for (int j = 0; j < cloth.points[0].size(); j++) {
					auto& point = cloth.points[i][j];
					// Assuming point coordinates are in the range [-1, 1] after transformations
					float t_point_x = point.x * 0.03f;
					float t_point_y = point.y * 0.03f;

					float dx = t_point_x - norm_mouse_x;
					float dy = t_point_y - norm_mouse_y;

					float dist = sqrt(dx * dx + dy * dy);

					if (dist < min_dist && dist <= max_radius) {
						min_dist = dist;
						closest_point = std::make_pair(i, j);
					}
				}
			}

			if (closest_point) {
				auto [i, j] = *closest_point;
				cloth.points[i][j].ext_m += 10.0f;
			}
		}
		else if (action == GLFW_RELEASE) {
			if (closest_point) {
				auto [i, j] = *closest_point;
				cloth.points[i][j].ext_m = 0.0f;
				closest_point.reset();
			}
		}
	}
	else if (button == GLFW_MOUSE_BUTTON_LEFT) {
		if (action == GLFW_PRESS)
		{
			moving_point = std::nullopt;
			float min_dist = std::numeric_limits<float>::max();
			for (int i = 0; i < cloth.points.size(); i++)
			{
				for (int j = 0; j < cloth.points[0].size(); j++)
				{
					auto& point = cloth.points[i][j];

					float t_point_x = point.x * 0.03f;
					float t_point_y = point.y * 0.03f;

					float dx = t_point_x - norm_mouse_x;
					float dy = t_point_y - norm_mouse_y;

					float dist = sqrt(dx * dx + dy * dy);

					if (dist < min_dist && dist <= max_radius) {
						min_dist = dist;
						moving_point = std::make_pair(i, j);
						cloth.points[i][j].fixed = true;
					}
				}
			}
		} else if (action == GLFW_RELEASE) {
			if (moving_point) {
				auto [i, j] = *moving_point;
				if (!cloth.points[i][j].static_point) {
					cloth.points[i][j].fixed = false;
				}
				moving_point.reset();
			}
        }
	}
}

void cursor_position_callback(GLFWwindow* window, double xpos, double ypos) {
	mouse_x = xpos;
	mouse_y = ypos;

	if (moving_point) {
		// Normalize mouse positions to range [-1, 1]
		float norm_mouse_x = (mouse_x / window_width) * 2.0f - 1.0f;
		float norm_mouse_y = -((mouse_y / window_height) * 2.0f - 1.0f);

		// Adjust for aspect ratio
		float aspect_ratio = (float)window_width / (float)window_height;
		norm_mouse_x *= aspect_ratio;

		// Convert normalized coordinates to world coordinates
		float new_x = norm_mouse_x / 0.03f;
		float new_y = norm_mouse_y / 0.03f;

		auto [i, j] = *moving_point;
		cloth.points[i][j].x = new_x;
		cloth.points[i][j].y = new_y;
	}
}

// Function to read shader source code from a file
char* read_shader_src(const std::string& path) {
	std::ifstream file(path);
	std::stringstream buffer;
	buffer << file.rdbuf();
	std::string content = buffer.str();

	char* src = new char[content.size() + 1];  // Allocate memory for the source code
	std::copy(content.begin(), content.end(), src);
	src[content.size()] = '\0';  // Null-terminate the string

	return src;
}

void simulate_CUDA(float dt) {
	std::vector<Point> flat_points = flatten_points(cloth.points, cloth.points.size(), cloth.points[0].size());

	simulate_kernel(flat_points.data(), cloth.springs.data(), N, M, cloth.springs.size(), dt, cloth.g, cloth.g_on, cloth.m);

	unflatten_points(cloth.points, flat_points, cloth.points.size(), cloth.points[0].size());
}

std::vector<Point> flatten_points(const std::vector<std::vector<Point>>& points, int N, int M) {
	std::vector<Point> flattenedPoints;
	flattenedPoints.reserve(N * M);

	for (const auto& row : points) {
		for (const auto& point : row) {
			flattenedPoints.push_back(point);
		}
	}

	return flattenedPoints;
}

void unflatten_points(std::vector<std::vector<Point>>& points, const std::vector<Point>& flattenedPoints, int N, int M) {
	for (int i = 0; i < N; i++) {
		for (int j = 0; j < M; j++) {
			points[i][j] = flattenedPoints[i * M + j];
		}
	}
}
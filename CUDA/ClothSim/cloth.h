#pragma once

#include <vector>
#include <cmath>
#include <cstdlib>

struct Point {
	float x, y;
	float vx, vy;
	float ax, ay;
	bool fixed;
	float ext_m;

	Point(float x, float y, float vx = 0.0f, float vy = 0.0f, float ax = 0.0f, float ay = 0.0f, bool fixed = false, float ext_m = 0.0f)
		: x(x), y(y), vx(vx), vy(vy), ax(ax), ay(ay), fixed(fixed), ext_m(ext_m) {}
};

struct Spring {
	std::pair<int, int> p1, p2;
	float rest_length;
	float spring_coeff;
	float damp_coeff;

	Spring(std::pair<int, int> p1, std::pair<int, int> p2, float rest_length, float spring_coeff, float damp_coeff)
		: p1(p1), p2(p2), rest_length(rest_length), spring_coeff(spring_coeff), damp_coeff(damp_coeff) {}
	
};

class Cloth {
public:
	Cloth(int n, int m);

	void simulate(float dt);

	std::vector<std::vector<Point>> points;
	std::vector<Spring> springs;

	bool g_on;
	float g;
	float m;
};
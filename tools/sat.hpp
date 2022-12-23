#pragma once

#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <sstream>
#include <tuple>
#include <vector>

enum class TriBool : uint8_t {
	None  = 0x00,
	False = 0x01,
	True  = 0x02,
};

using Var           = uint32_t;
using AssignmentVec = std::vector<TriBool>;

class Lit {
public:
	constexpr Lit(const int32_t dimacs_lit) : lit(0) {
		assert(dimacs_lit != 0);
		const bool is_pos = dimacs_lit > 0;
		const Var  var    = is_pos ? dimacs_lit : -dimacs_lit;
		lit               = var | (is_pos ? pos_mask : 0);
	}

	constexpr Lit(const Var var, const bool is_pos) : lit(var | (is_pos ? pos_mask : 0)) {
		assert(var != 0);
	}

	constexpr Var     var() const { return lit & ~pos_mask; }
	constexpr bool    is_pos() const { return lit & pos_mask; }
	constexpr TriBool tri_bool() const { return is_pos() ? TriBool::True : TriBool::False; }

	bool sat_by(const AssignmentVec& assignment) const { return assignment[var()] == tri_bool(); }

	bool operator==(const Lit& rhs) const { return lit == rhs.lit; }

	friend std::ostream& operator<<(std::ostream& out, const Lit& lit) {
		if (!lit.is_pos())
			out << '-';
		return out << lit.var();
	}

	static Lit make_lit(const std::string& s) {
		bool      is_pos = (s[0] != '-');
		const Var val    = std::stoi(is_pos ? s : s.substr(1));
		return Lit{val, is_pos};
	}

private:
	static constexpr Var pos_mask = 0x80000000;
	Var                  lit;
};

using Clause     = std::vector<Lit>;
using Formula    = std::vector<Clause>;
using Assignment = std::vector<Lit>;

inline std::vector<std::string> split(std::string const& input) {
	std::istringstream       buffer(input);
	std::vector<std::string> ret{std::istream_iterator<std::string>(buffer), {}};
	return ret;
}

inline std::tuple<Formula, std::size_t> parse_formula(std::ifstream& fs) {
	Formula     formula;
	std::size_t num_variables = 0;
	std::size_t num_clauses   = 0;

	std::string line;
	// Read comment/problem statement
	while (std::getline(fs, line)) {
		if (line.size() == 0 || line[0] == 'c')
			continue;

		if (line[0] == 'p') {
			const auto parts = split(line);
			num_variables    = std::stoi(parts[2]);
			num_clauses      = std::stoi(parts[3]);
			break;
		}

		throw std::runtime_error("Invalid line in CNF");
	}

	formula.reserve(num_clauses);

	int    lit_int;
	Clause clause;
	while (fs >> lit_int) {
		if (lit_int == 0) {
			formula.emplace_back(std::move(clause));
			clause.clear();
			continue;
		}

		Lit lit{lit_int};
		assert(lit.var() <= num_variables);
		clause.emplace_back(lit);
	}

	if (!clause.empty())
		formula.emplace_back(std::move(clause));

	assert(formula.size() == num_clauses);
	return std::make_pair(std::move(formula), num_variables);
}

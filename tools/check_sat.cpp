// A sketch of a fast unverified SAT proof checker
// This version streams clauses in and checks them as they are read, rather than
// storing them in memory. This makes parsing faster and prevents the need to
// store all the clauses in memory. Additionally the proof file is read directly
// into the assignment vector

#include <cassert>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <sstream>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

enum class TriBool : uint8_t {
	None  = 0x00,
	False = 0x01,
	True  = 0x02,
};

using Var        = uint32_t;
using Assignment = std::vector<TriBool>;

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

	constexpr Var     raw() const { return lit; }
	constexpr Var     var() const { return lit & ~pos_mask; }
	constexpr bool    is_pos() const { return lit & pos_mask; }
	constexpr TriBool tri_bool() const { return is_pos() ? TriBool::True : TriBool::False; }
	constexpr bool    is_zero() const { return var() == 0; }

	bool sat_by(const Assignment& assignment) const { return assignment[var()] == tri_bool(); }

	bool operator==(const Lit& rhs) const { return lit == rhs.lit; }
	bool operator<(const Lit& rhs) const { return var() < rhs.var(); }

	friend std::ostream& operator<<(std::ostream& out, const Lit& lit) {
		if (!lit.is_pos())
			out << '-';
		return out << lit.var();
	}

private:
	static constexpr Var pos_mask = 0x80000000;
	Var                  lit;
};

inline std::vector<std::string> split(std::string const& input) {
	std::istringstream       buffer(input);
	std::vector<std::string> ret{std::istream_iterator<std::string>(buffer), {}};
	return ret;
}

inline bool parse_and_check_formula(std::ifstream& fs, Assignment& assignment) {
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

	if (assignment.size() < num_variables + 1) {
		assignment.resize(num_variables + 1, TriBool::None);
	}

	assert(assignment.size() == num_variables + 1);

	int32_t lit_int    = 0;
	size_t  num_zeros  = 0;
	bool    clause_sat = false;
	while (fs >> lit_int) {
		if (lit_int == 0) {
			num_zeros++;
			if (!clause_sat) {
				return false;
			}
			clause_sat = false;
			continue;
		}

		Lit lit{lit_int};
		assert(lit.var() <= num_variables);
		if (!clause_sat && lit.sat_by(assignment))
			clause_sat = true;
	}

	assert(lit_int == 0);
	assert(num_zeros == num_clauses);
	return true;
}

Assignment parse_assignment(std::ifstream& fs) {
	Assignment assignment;

	char        next_char;
	std::string line;
	while (std::getline(fs, line)) {
		if (line.empty())
			continue;

		switch (line[0]) {
			case 'c': continue;
			case 's': {
				assert(line.find(" SATISFIABLE") != std::string::npos);
				continue;
			}
			case 'v': {
				std::stringstream ss{line};
				ss.seekg(1, ss.beg);
				int32_t lit_int = 0;
				while (ss >> lit_int) {
					if (lit_int == 0)
						break;
					const Lit  lit{lit_int};
					const auto var = lit.var();
					if (var >= assignment.size()) {
						assignment.resize(var + 1, TriBool::None);
					}
					assert(assignment[var] == TriBool::None || assignment[var] == lit.tri_bool());
					assignment[var] = lit.tri_bool();
				}
				continue;
			}
			default: throw std::runtime_error("Invalid line in CNF");
		}
	}

	while (fs.get(next_char)) {}
	return assignment;
}

int main(int argc, char* argv[]) {
	if (argc != 3)
		return EXIT_FAILURE;

	const auto    start = std::chrono::high_resolution_clock::now();
	std::ifstream proof;
	proof.open(argv[2]);
	auto assignment = parse_assignment(proof);
	proof.close();
	const auto done_proof = std::chrono::high_resolution_clock::now();

	std::ifstream dimacs;
	dimacs.open(argv[1]);
	const auto sat = parse_and_check_formula(dimacs, assignment);
	dimacs.close();
	const auto done_verification = std::chrono::high_resolution_clock::now();

	if (sat)
		std::cout << "VERIFIED" << std::endl;
	else
		std::cout << "NOT VERIFIED" << std::endl;

	const auto proof_parse_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_proof - start);
	const auto verification_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_verification - done_proof);
	std::cout << "Proof parsing took " << proof_parse_time.count() << " milliseconds" << std::endl;
	std::cout << "DIMACS parsing and verification took " << verification_time.count()
	          << " milliseconds" << std::endl;
}

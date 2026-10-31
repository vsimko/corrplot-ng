#include <Rcpp.h>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <string>

namespace {
std::string json_escape(const std::string& value) {
  std::ostringstream out;
  for (unsigned char ch : value) {
    switch (ch) {
    case '"': out << "\\\""; break;
    case '\\': out << "\\\\"; break;
    case '\b': out << "\\b"; break;
    case '\f': out << "\\f"; break;
    case '\n': out << "\\n"; break;
    case '\r': out << "\\r"; break;
    case '\t': out << "\\t"; break;
    default:
      if (ch < 0x20) {
        out << "\\u" << std::hex << std::setw(4) << std::setfill('0')
            << static_cast<int>(ch) << std::dec;
      } else {
        out << static_cast<char>(ch);
      }
    }
  }
  return out.str();
}

std::string number(double value) {
  std::ostringstream out;
  out << std::setprecision(15) << value;
  return out.str();
}
} // namespace

// [[Rcpp::export]]
std::string build_scene_json_cpp(Rcpp::CharacterVector labels,
                                 Rcpp::NumericMatrix matrix,
                                 std::string annotation = "") {
  const int n = labels.size();
  if (n == 0 || matrix.nrow() != n || matrix.ncol() != n) {
    Rcpp::stop("labels and matrix dimensions must agree and be non-empty");
  }

  const double cell = 84.0;
  const double left = 104.0;
  const double top = 76.0;
  const double width = left + n * cell + 24.0;
  const double height = top + n * cell + 64.0;

  std::ostringstream out;
  out << "{\"schema_version\":1,\"kind\":\"correlation_plot\","
      << "\"width\":" << number(width) << ",\"height\":" << number(height)
      << ",\"labels\":[";
  for (int i = 0; i < n; ++i) {
    if (i) out << ',';
    if (labels[i] == NA_STRING) Rcpp::stop("labels cannot be NA");
    const std::string label = Rcpp::as<std::string>(labels[i]);
    out << "{\"text\":\"" << json_escape(label)
        << "\",\"row_x\":92,\"row_y\":" << number(top + i * cell + cell / 2.0)
        << ",\"column_x\":" << number(left + i * cell + cell / 2.0)
        << ",\"column_y\":60}";
  }
  out << "],\"cells\":[";
  bool first = true;
  for (int row = 0; row < n; ++row) {
    for (int column = 0; column < n; ++column) {
      const double value = matrix(row, column);
      if (!R_finite(value) && !R_IsNA(value)) {
        Rcpp::stop("matrix values must be finite, NA, or between -1 and 1");
      }
      if (!R_IsNA(value) && (value < -1.0 || value > 1.0)) {
        Rcpp::stop("matrix values must be finite, NA, or between -1 and 1");
      }
      if (!first) out << ',';
      first = false;
      out << "{\"row\":" << row << ",\"column\":" << column
          << ",\"value\":";
      if (R_IsNA(value)) out << "null";
      else out << number(value);
      out << ",\"x\":" << number(left + column * cell)
          << ",\"y\":" << number(top + row * cell)
          << ",\"center_x\":" << number(left + column * cell + cell / 2.0)
          << ",\"center_y\":" << number(top + row * cell + cell / 2.0)
          << ",\"size\":84}";
    }
  }
  out << "],\"annotation\":\"" << json_escape(annotation)
      << "\",\"style\":{\"positive_color\":\"#2166ac\","
      << "\"negative_color\":\"#b2182b\",\"zero_color\":\"#bdbdbd\","
      << "\"missing_fill\":\"#f2f2f2\",\"grid_color\":\"#e0e0e0\"}}";
  return out.str();
}

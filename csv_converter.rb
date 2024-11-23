class CsvConverter < Formula
  desc "Tool to convert a CSV file into a new format"
  homepage "https://github.com/the-working-party/csv_converter"
  url "https://github.com/the-working-party/csv_converter/archive/refs/tags/v1.0.1.tar.gz"
  sha256 "40abb007b829dd2e0e2cc0598a00c512ad8dd820eb31b2f9b0a8f2b89fa08d79"
  license "MIT"
  head "https://github.com/the-working-party/csv_converter.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"csv_converter", "--version"
  end
end

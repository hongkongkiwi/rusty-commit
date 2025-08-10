Name:           rusty-commit
Version:        VERSION
Release:        1%{?dist}
Summary:        Rust-powered AI commit message generator
License:        MIT
URL:            https://github.com/hongkongkiwi/rusty-commit
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo
Requires:       git

%description
Blazing-fast commit messages powered by AI and written in Rust.
Supports 16+ AI providers including Anthropic, OpenAI, OpenRouter,
Groq, DeepSeek, and more. Features interactive authentication,
secure credential storage, and full OpenCommit compatibility.

%prep
%setup -q

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/bin
install -m 755 target/release/rco $RPM_BUILD_ROOT/usr/bin/rco

%files
/usr/bin/rco

%changelog
* VERSION_DATE Rusty Commit Contributors - VERSION
- Release VERSION
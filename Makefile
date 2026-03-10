PREFIX ?= /usr/local

.PHONY: install uninstall test lint clean setup-bats

install:
	install -d $(PREFIX)/bin
	install -m 755 moltctrl $(PREFIX)/bin/moltctrl
	install -d $(PREFIX)/share/moltctrl/profiles
	install -m 644 profiles/openclaw.yml.tmpl $(PREFIX)/share/moltctrl/profiles/

uninstall:
	rm -f $(PREFIX)/bin/moltctrl
	rm -rf $(PREFIX)/share/moltctrl

setup-bats:
	@if [ ! -d tests/test_helper/bats-core ]; then \
		echo "Installing bats-core..."; \
		git clone --depth 1 https://github.com/bats-core/bats-core.git tests/test_helper/bats-core; \
	fi
	@if [ ! -d tests/test_helper/bats-support ]; then \
		echo "Installing bats-support..."; \
		git clone --depth 1 https://github.com/bats-core/bats-support.git tests/test_helper/bats-support; \
	fi
	@if [ ! -d tests/test_helper/bats-assert ]; then \
		echo "Installing bats-assert..."; \
		git clone --depth 1 https://github.com/bats-core/bats-assert.git tests/test_helper/bats-assert; \
	fi

test: setup-bats
	tests/test_helper/bats-core/bin/bats tests/

lint:
	shellcheck -x moltctrl install.sh

clean:
	rm -rf tests/test_helper/bats-core tests/test_helper/bats-support tests/test_helper/bats-assert

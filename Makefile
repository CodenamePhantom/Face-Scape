BINARY_NAME = face-scape
PAM_NAME = libpam_facescape.so
WORKSPACE_BINARY = target/release/$(BINARY_NAME)
WORKSPACE_PAM = target/release/$(PAM_NAME)

PREFIX = /usr/local
BIN_DIR = $(PREFIX)/bin
PAM_DIR = /lib/security
ETC_DIR = /etc/facescape
SYSTEMD_DIR = /etc/systemd/system
PAM_D_DIR = /etc/pam.d

.PHONY: all build install daemon clean uninstall rollback-pam

all: build

build:
	@echo "[FaceScape Build] Compiling release binaries..."
	cargo build --release
install: build
	@echo "[FaceScape Install] Provisioning system environments..."
	sudo mkdir -p $(ETC_DIR)
	sudo chmod 755 $(ETC_DIR)

	@echo "[FaceScape Install] Deploying components to system paths..."
	sudo cp $(WORKSPACE_BINARY) $(BIN_DIR)/$(BINARY_NAME)
	sudo chmod 755 $(BIN_DIR)/$(BINARY_NAME)

	sudo cp $(WORKSPACE_PAM) $(PAM_DIR)/pam_facescape.so
	sudo chmod 755 $(PAM_DIR)/pam_facescape.so

	@echo "[FaceScape Install] Modifying active PAM configuration files..."
	sudo cp $(PAM_D_DIR)/sudo $(PAM_D_DIR)/sudo.bak

	echo "#%PAM-1.0" | sudo tee $(PAM_D_DIR)sudo.tmp > /dev/null
	echo "auth				sufficient		pam_facescape.so" | sudo tee -a $(PAM_D_DIR)/sudo.tmp > /dev/null
	sudo grep -v "^#%PAM-1.0" $(PAM_D_DIR)/sudo >> $(PAM_D_DIR)/sudo.tmp
	sudo mv $(PAM_D_DIR)/sudo.tmp $(PAM_D_DIR)/sudo

	@echo "[FaceScape Install] Deployment complete."

daemon: install
	@echo "[FaceScape Daemon] Generating Systemd unit service..."
	echo "[Unit]" | sudo tee $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "Description=FaceScape Biometric Authentication Memory Arena Daemon" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "After=network.target" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "[Service]" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "Type=simple" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "ExecStart=$(BIN_DIR)/$(BINARY_NAME) manage start" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "ExecStop=$(BIN_DIR)/$(BINARY_NAME) manage stop" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "Restart=always" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "User=root" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "[Install]" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null
	echo "WantedBy=multi-user.target" | sudo tee -a $(SYSTEMD_DIR)/facescape.service > /dev/null

	@echo "[FaceScape Daemon] Activating background initialization matrix daemon..."
	sudo systemctl daemon-reload
	sudo systemctl enable facescape.service
	sudo systemctl start facescape.service
	@sudo systemctl status facescape.service --no-pager

clean:
	@echo "[FaceScape Clean] Purging local workspace builds..."
	cargo clean

uninstall:
	@echo "[FaceScape Uninstall] Stripping assets from environment..."
	sudo systemctl stop facescape.service || true
	sudo systemctl disable facescape.service || true
	sudo rm -f $(SYSTEMD_DIR)/facescape.service
	sudo systemctl daemon-reload
	sudo rm -f $(BIN_DIR)/$(BINARY_NAME)
	sudo rm -f $(LIB_DIR)/$(BINARY_NAME)
	sudo rm -f $(PAM_DIR)/pam_facescape.so
	sudo mv $(PAM_D_DIR)/sudo.bak $(PAM_D_DIR)/sudo
	@echo "[FaceScape Uninstall] Notice: Enrolled profiles inside $(ETC_DIR) left intact to protect data."

rollback-pam:
	@echo "[FaceScape Rollback] rolling back PAM integrations..."
	sudo mv $(PAM_D_DIR)/sudo.bak $(PAM_D_DIR)/sudo

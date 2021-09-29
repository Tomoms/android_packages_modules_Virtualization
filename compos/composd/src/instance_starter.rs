/*
 * Copyright (C) 2021 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Responsible for validating and starting an existing instance of the CompOS VM, or creating and
//! starting a new instance if necessary.

use android_system_virtualizationservice::aidl::android::system::virtualizationservice::{
    IVirtualizationService::IVirtualizationService, PartitionType::PartitionType,
};
use anyhow::{bail, Context, Result};
use compos_aidl_interface::aidl::com::android::compos::ICompOsService::ICompOsService;
use compos_aidl_interface::binder::{ParcelFileDescriptor, Strong};
use compos_common::compos_client::VmInstance;
use compos_common::{
    COMPOS_DATA_ROOT, INSTANCE_IMAGE_FILE, PRIVATE_KEY_BLOB_FILE, PUBLIC_KEY_FILE,
};
use log::{info, warn};
use std::fs;
use std::path::{Path, PathBuf};

pub struct CompOsInstance {
    #[allow(dead_code)] // Keeps VirtualizationService & the VM alive
    vm_instance: VmInstance,
    service: Strong<dyn ICompOsService>,
}

impl CompOsInstance {
    pub fn get_service(&self) -> Strong<dyn ICompOsService> {
        self.service.clone()
    }
}

pub struct InstanceStarter {
    instance_name: String,
    instance_root: PathBuf,
    instance_image: PathBuf,
    key_blob: PathBuf,
    public_key: PathBuf,
}

impl InstanceStarter {
    pub fn new(instance_name: &str) -> Self {
        let instance_root = Path::new(COMPOS_DATA_ROOT).join(instance_name);
        let instant_root_path = instance_root.as_path();
        let instance_image = instant_root_path.join(INSTANCE_IMAGE_FILE);
        let key_blob = instant_root_path.join(PRIVATE_KEY_BLOB_FILE);
        let public_key = instant_root_path.join(PUBLIC_KEY_FILE);
        Self {
            instance_name: instance_name.to_owned(),
            instance_root,
            instance_image,
            key_blob,
            public_key,
        }
    }

    pub fn create_or_start_instance(
        &self,
        service: &dyn IVirtualizationService,
    ) -> Result<CompOsInstance> {
        let compos_instance = self.start_existing_instance();
        match compos_instance {
            Ok(_) => return compos_instance,
            Err(e) => warn!("Failed to start {}: {}", self.instance_name, e),
        }

        self.start_new_instance(service)
    }

    fn start_existing_instance(&self) -> Result<CompOsInstance> {
        // No point even trying if the files we need aren't there.
        self.check_files_exist()?;

        let key_blob = fs::read(&self.key_blob).context("Reading private key blob")?;
        let public_key = fs::read(&self.public_key).context("Reading public key")?;

        let vm_instance = VmInstance::start(&self.instance_image).context("Starting VM")?;
        let service = vm_instance.get_service().context("Connecting to CompOS")?;

        if !service.verifySigningKey(&key_blob, &public_key).context("Verifying key pair")? {
            bail!("Key pair invalid");
        }

        // If we get this far then the instance image is valid in the current context (e.g. the
        // current set of APEXes) and the key blob can be successfully decrypted by the VM. So the
        // files have not been tampered with and we're good to go.

        service.initializeSigningKey(&key_blob).context("Loading signing key")?;

        Ok(CompOsInstance { vm_instance, service })
    }

    fn start_new_instance(
        &self,
        virtualization_service: &dyn IVirtualizationService,
    ) -> Result<CompOsInstance> {
        info!("Creating {} CompOs instance", self.instance_name);

        // Ignore failure here - the directory may already exist.
        let _ = fs::create_dir(&self.instance_root);

        self.create_instance_image(virtualization_service)?;

        let vm_instance = VmInstance::start(&self.instance_image).context("Starting VM")?;
        let service = vm_instance.get_service().context("Connecting to CompOS")?;

        let key_data = service.generateSigningKey().context("Generating signing key")?;
        fs::write(&self.key_blob, &key_data.keyBlob).context("Writing key blob")?;
        // TODO: Extract public key from cert
        fs::write(&self.public_key, &key_data.certificate).context("Writing public key")?;

        // We don't need to verify the key, since we just generated it and have it in memory.

        service.initializeSigningKey(&key_data.keyBlob).context("Loading signing key")?;

        Ok(CompOsInstance { vm_instance, service })
    }

    fn create_instance_image(
        &self,
        virtualization_service: &dyn IVirtualizationService,
    ) -> Result<()> {
        let instance_image = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.instance_image)
            .context("Creating instance image file")?;
        let instance_image = ParcelFileDescriptor::new(instance_image);
        // TODO: Where does this number come from?
        let size = 10 * 1024 * 1024;
        virtualization_service
            .initializeWritablePartition(&instance_image, size, PartitionType::ANDROID_VM_INSTANCE)
            .context("Writing instance image file")?;
        Ok(())
    }

    fn check_files_exist(&self) -> Result<()> {
        if !self.instance_root.is_dir() {
            bail!("Directory {} not found", self.instance_root.display())
        };
        Self::check_file_exists(&self.instance_image)?;
        Self::check_file_exists(&self.key_blob)?;
        Self::check_file_exists(&self.public_key)?;
        Ok(())
    }

    fn check_file_exists(file: &Path) -> Result<()> {
        if !file.is_file() {
            bail!("File {} not found", file.display())
        };
        Ok(())
    }
}
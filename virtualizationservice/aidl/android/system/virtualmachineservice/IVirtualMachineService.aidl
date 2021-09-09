/*
 * Copyright 2021 The Android Open Source Project
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
package android.system.virtualmachineservice;

/** {@hide} */
interface IVirtualMachineService {
    /**
     * Port number that VirtualMachineService listens on connections from the guest VMs for the
     * payload input and output.
     */
    const int VM_STREAM_SERVICE_PORT = 3000;

    /**
     * Port number that VirtualMachineService listens on connections from the guest VMs for the
     * VirtualMachineService binder service.
     */
    const int VM_BINDER_SERVICE_PORT = 5000;

    /**
     * Notifies that the payload has started.
     * TODO(b/191845268): remove cid parameter
     */
    void notifyPayloadStarted(int cid);

    /**
     * Notifies that the payload is ready to serve.
     * TODO(b/191845268): remove cid parameter
     */
    void notifyPayloadReady(int cid);

    /**
     * Notifies that the payload has finished.
     * TODO(b/191845268): remove cid parameter
     */
    void notifyPayloadFinished(int cid, int exitCode);
}
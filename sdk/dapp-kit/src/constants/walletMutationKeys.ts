// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import type { MutationKey } from '@tanstack/react-query';

export const walletMutationKeys = {
	all: { baseScope: 'wallet' },
	connectWallet: formMutationKeyFn('connect-wallet'),
};

function formMutationKeyFn(baseEntity: string) {
	return function mutationKeyFn(additionalKeys: MutationKey = []) {
		return [{ ...walletMutationKeys.all, baseEntity }, ...additionalKeys];
	};
}
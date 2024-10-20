export type Alias = string;
export type VoterToken = string;

export interface VoterLoginDetails {
	alias: Alias;
	token: VoterToken;
}

export interface VotingMeta {
	hideVoteCount: boolean;
	seats: number;
	name: string;
	description: string;
}

export type Regexable<T> = {
	[K in keyof T]: T[K] extends string | number ? T[K] | RegExp : T[K];
};

export type Locatable<T> = Partial<Regexable<T>>;

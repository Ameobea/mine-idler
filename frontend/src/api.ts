import { type Code, ConnectError, createPromiseClient, type Interceptor } from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { PUBLIC_API_BASE_URL } from '$env/static/public';

import { MinePrivateService, MinePublicService } from './protos/mine_connect';

const BaseURL = PUBLIC_API_BASE_URL ?? 'http://localhost:5900';

export const ImageBaseURL = 'https://mine-idler.b-cdn.net/';

let SessionToken: string | null = globalThis.localStorage ? localStorage.getItem('sessionToken') : null;

const setSessionToken = (token: string) => {
  SessionToken = token;
  localStorage.setItem('sessionToken', token);
};

export const getIsLoggedIn = () => !!SessionToken;

const customFetch: typeof globalThis.fetch = async (...args) => {
  const res = await fetch(...args);
  const code = res.headers.get('grpc-status');
  if (code === '0' || !code) {
    return res;
  }

  let errMsg = res.headers.get('grpc-message');
  errMsg = errMsg ? decodeURIComponent(errMsg) : 'Unknown error';
  throw new ConnectError(errMsg, +code as Code);
};

const authInterceptor: Interceptor = (next) => async (req) => {
  if (!SessionToken) {
    throw new Error('Not logged in');
  }

  req.header.set('authorization', SessionToken);
  return await next(req);
};

export const PublicClient = createPromiseClient(
  MinePublicService,
  createGrpcWebTransport({ baseUrl: BaseURL, fetch: customFetch })
);
export const PrivateClient = createPromiseClient(
  MinePrivateService,
  createGrpcWebTransport({
    baseUrl: BaseURL,
    fetch: customFetch,
    interceptors: [authInterceptor],
  })
);

export const login = async (username: string, password: string) =>
  setSessionToken((await PublicClient.login({ username, password })).sessionToken);

export const register = async (username: string, password: string) =>
  setSessionToken((await PublicClient.register({ username, password })).sessionToken);

// @generated by protoc-gen-connect-es v1.1.2 with parameter "target=ts"
// @generated from file protos/mine.proto (package mine, syntax proto3)
/* eslint-disable */
// @ts-nocheck

import { GetAccountRequest, GetAccountResponse, GetBaseRequest, GetBaseResponse, GetGambleLocationsRequest, GetGambleLocationsResponse, GetHiscoresRequest, GetHiscoresResponse, GetInventoryRequest, GetInventoryResponse, GetItemDescriptorsRequest, GetItemDescriptorsResponse, GetMineLocationsRequest, GetMineLocationsResponse, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, StartMiningRequest, StartMiningResponse, StopMiningRequest, StopMiningResponse, UpgradeBaseRequest, UpgradeBaseResponse } from "./mine_pb.js";
import { MethodKind } from "@bufbuild/protobuf";

/**
 * Unauthenticated service
 *
 * @generated from service mine.MinePublicService
 */
export const MinePublicService = {
  typeName: "mine.MinePublicService",
  methods: {
    /**
     * Auth
     *
     * @generated from rpc mine.MinePublicService.Login
     */
    login: {
      name: "Login",
      I: LoginRequest,
      O: LoginResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePublicService.Register
     */
    register: {
      name: "Register",
      I: RegisterRequest,
      O: RegisterResponse,
      kind: MethodKind.Unary,
    },
    /**
     * Community
     *
     * @generated from rpc mine.MinePublicService.GetHiscores
     */
    getHiscores: {
      name: "GetHiscores",
      I: GetHiscoresRequest,
      O: GetHiscoresResponse,
      kind: MethodKind.Unary,
    },
  }
} as const;

/**
 * Authenticated service
 *
 * @generated from service mine.MinePrivateService
 */
export const MinePrivateService = {
  typeName: "mine.MinePrivateService",
  methods: {
    /**
     * General
     *
     * @generated from rpc mine.MinePrivateService.GetItemDescriptors
     */
    getItemDescriptors: {
      name: "GetItemDescriptors",
      I: GetItemDescriptorsRequest,
      O: GetItemDescriptorsResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePrivateService.GetMineLocations
     */
    getMineLocations: {
      name: "GetMineLocations",
      I: GetMineLocationsRequest,
      O: GetMineLocationsResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePrivateService.GetGambleLocations
     */
    getGambleLocations: {
      name: "GetGambleLocations",
      I: GetGambleLocationsRequest,
      O: GetGambleLocationsResponse,
      kind: MethodKind.Unary,
    },
    /**
     * Account
     *
     * @generated from rpc mine.MinePrivateService.GetAccount
     */
    getAccount: {
      name: "GetAccount",
      I: GetAccountRequest,
      O: GetAccountResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePrivateService.GetInventory
     */
    getInventory: {
      name: "GetInventory",
      I: GetInventoryRequest,
      O: GetInventoryResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePrivateService.GetBase
     */
    getBase: {
      name: "GetBase",
      I: GetBaseRequest,
      O: GetBaseResponse,
      kind: MethodKind.Unary,
    },
    /**
     * Gameplay
     *
     * @generated from rpc mine.MinePrivateService.StartMining
     */
    startMining: {
      name: "StartMining",
      I: StartMiningRequest,
      O: StartMiningResponse,
      kind: MethodKind.ServerStreaming,
    },
    /**
     * @generated from rpc mine.MinePrivateService.StopMining
     */
    stopMining: {
      name: "StopMining",
      I: StopMiningRequest,
      O: StopMiningResponse,
      kind: MethodKind.Unary,
    },
    /**
     * @generated from rpc mine.MinePrivateService.UpgradeBase
     */
    upgradeBase: {
      name: "UpgradeBase",
      I: UpgradeBaseRequest,
      O: UpgradeBaseResponse,
      kind: MethodKind.Unary,
    },
  }
} as const;

